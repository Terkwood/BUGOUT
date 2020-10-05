mod xadd;
mod xread;

pub use xadd::*;
pub use xread::*;

use crate::components::Components;
use crate::game_lobby::GameLobbyOps;
use crate::repo::EntryIdType;
use crate::PUBLIC_GAME_BOARD_SIZE;
use core_model::*;
use lobby_model::api::*;
use lobby_model::*;

use log::{error, warn};
use redis_streams::XReadEntryId;

pub fn process(reg: &Components) {
    loop {
        match reg.entry_id_repo.fetch_all() {
            Ok(all_eids) => match reg.xread.xread_sorted(all_eids) {
                Ok(xrr) => {
                    for (eid, data) in xrr {
                        consume(eid, &data, &reg);
                        increment(eid, data, reg);
                    }
                }
                Err(e) => error!("Stream err {:?}", e),
            },
            Err(e) => error!("Failed to fetch EIDs {:?}", e),
        }
    }
}

fn consume(_eid: XReadEntryId, event: &StreamInput, reg: &Components) {
    match event {
        StreamInput::FPG(fpg) => consume_fpg(fpg, reg),
        StreamInput::CG(cg) => consume_cg(cg, reg),
        StreamInput::JPG(jpg) => consume_jpg(jpg, reg),
        StreamInput::SD(sd) => consume_sd(sd, reg),
    }
}

fn consume_fpg(fpg: &FindPublicGame, reg: &Components) {
    let session_id = &fpg.session_id;
    if let Ok(lobby) = reg.game_lobby_repo.get() {
        if let Some(queued) = lobby
            .games
            .iter()
            .find(|g| g.visibility == Visibility::Public)
        {
            ready_xadd(session_id, &lobby, queued, reg)
        } else {
            let game_id = GameId::new();
            if let Err(_) = reg.game_lobby_repo.put(lobby.open(Game {
                board_size: PUBLIC_GAME_BOARD_SIZE,
                creator: session_id.clone(),
                visibility: Visibility::Public,
                game_id: game_id.clone(),
            })) {
                error!("game lobby write F2");
            } else {
                if let Err(_) = reg.xadd.xadd(StreamOutput::WFO(WaitForOpponent {
                    event_id: EventId::new(),
                    game_id,
                    session_id: session_id.clone(),
                })) {
                    error!("XADD: Wait for oppo")
                }
            }
        }
    } else {
        error!("Failed to fetch game lobby: FPG")
    }
}

fn consume_cg(cg: &CreateGame, reg: &Components) {
    let session_id = &cg.session_id;
    let game_id = cg.game_id.clone().unwrap_or(GameId::new());
    if let Ok(lobby) = reg.game_lobby_repo.get() {
        let updated_gl = lobby.open(Game {
            game_id: game_id.clone(),
            board_size: cg.board_size,
            creator: session_id.clone(),
            visibility: cg.visibility,
        });
        if let Err(_) = reg.game_lobby_repo.put(updated_gl) {
            error!("game lobby write F1");
        } else {
            if let Err(_) = reg.xadd.xadd(StreamOutput::WFO(WaitForOpponent {
                game_id: game_id.clone(),
                session_id: session_id.clone(),
                event_id: EventId::new(),
            })) {
                error!("XADD Game ready")
            }
        }
    } else {
        error!("CG GAME REPO GET")
    }
}

/// Consumes the command to join a private game.
/// In the event that the game is invalid,
/// we will simply log a warning.
/// Consider implementing logic related to handling
/// private game rejection: https://github.com/Terkwood/BUGOUT/issues/304
fn consume_jpg(jpg: &JoinPrivateGame, reg: &Components) {
    if let Ok(lobby) = reg.game_lobby_repo.get() {
        if let Some(queued) = lobby
            .games
            .iter()
            .find(|g| g.visibility == Visibility::Private && g.game_id == jpg.game_id)
        {
            ready_xadd(&jpg.session_id, &lobby, queued, reg)
        } else {
            warn!("Ignoring game rejection event")
        }
    } else {
        error!("game lobby JPG get")
    }
}

fn consume_sd(sd: &SessionDisconnected, reg: &Components) {
    if let Ok(game_lobby) = reg.game_lobby_repo.get() {
        let u = game_lobby.abandon(&sd.session_id);
        if let Err(_) = reg.game_lobby_repo.put(u) {
            error!("game lobby write F1");
        }
    } else {
        error!("SD GAME REPO GET")
    }
}

fn ready_xadd(session_id: &SessionId, lobby: &GameLobby, queued: &Game, reg: &Components) {
    let updated_gl = lobby.ready(queued);
    if let Err(_) = reg.game_lobby_repo.put(updated_gl) {
        error!("game lobby write F1");
    } else {
        if let Err(_) = reg.xadd.xadd(StreamOutput::GR(GameReady {
            game_id: queued.game_id.clone(),
            event_id: EventId::new(),
            board_size: queued.board_size,
            sessions: (queued.creator.clone(), session_id.clone()),
        })) {
            error!("XADD Game ready")
        }
    }
}

fn increment(eid: XReadEntryId, event: StreamInput, reg: &Components) {
    if let Err(e) = reg.entry_id_repo.update(EntryIdType::from(event), eid) {
        error!("eid write {:?}", e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::components::Components;
    use crate::repo::*;
    use crossbeam_channel::{select, unbounded, Sender};
    use redis_streams::XReadEntryId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use std::thread;
    static FAKE_FPG_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_CG_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_JPG_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_SD_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_FPG_SEQ: AtomicU64 = AtomicU64::new(0);
    static FAKE_CG_SEQ: AtomicU64 = AtomicU64::new(0);
    static FAKE_JPG_SEQ: AtomicU64 = AtomicU64::new(0);
    static FAKE_SD_SEQ: AtomicU64 = AtomicU64::new(0);
    struct FakeEIDRepo {
        call_in: Sender<EIDRepoCalled>,
    }
    #[derive(Debug)]
    enum EIDRepoCalled {
        Fetch,
        Update(EntryIdType, XReadEntryId),
    }
    impl EntryIdRepo for FakeEIDRepo {
        fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
            self.call_in.send(EIDRepoCalled::Fetch).expect("send");
            Ok(AllEntryIds {
                find_public_game: XReadEntryId {
                    millis_time: FAKE_FPG_MILLIS.load(Ordering::Relaxed),
                    seq_no: FAKE_FPG_SEQ.load(Ordering::Relaxed),
                },
                create_game: XReadEntryId {
                    millis_time: FAKE_CG_MILLIS.load(Ordering::Relaxed),
                    seq_no: FAKE_CG_SEQ.load(Ordering::Relaxed),
                },
                join_private_game: XReadEntryId {
                    millis_time: FAKE_JPG_MILLIS.load(Ordering::Relaxed),
                    seq_no: FAKE_JPG_SEQ.load(Ordering::Relaxed),
                },
                session_disconnected: XReadEntryId {
                    millis_time: FAKE_SD_MILLIS.load(Ordering::Relaxed),
                    seq_no: FAKE_SD_SEQ.load(Ordering::Relaxed),
                },
            })
        }
        fn update(
            &self,
            eid_type: EntryIdType,
            eid: redis_streams::XReadEntryId,
        ) -> Result<(), WriteErr> {
            self.call_in
                .send(EIDRepoCalled::Update(eid_type, eid))
                .expect("send");
            match eid_type {
                EntryIdType::FindPublicGameCmd => {
                    FAKE_FPG_MILLIS.swap(eid.millis_time, Ordering::Relaxed);
                    FAKE_FPG_SEQ.swap(eid.seq_no, Ordering::Relaxed);
                }
                EntryIdType::CreateGameCmd => {
                    FAKE_CG_MILLIS.swap(eid.millis_time, Ordering::Relaxed);
                    FAKE_CG_SEQ.swap(eid.seq_no, Ordering::Relaxed);
                }
                EntryIdType::JoinPrivateGameCmd => {
                    FAKE_JPG_MILLIS.swap(eid.millis_time, Ordering::Relaxed);
                    FAKE_JPG_SEQ.swap(eid.seq_no, Ordering::Relaxed);
                }
                EntryIdType::SessionDisconnectedEv => {
                    FAKE_SD_MILLIS.swap(eid.millis_time, Ordering::Relaxed);
                    FAKE_SD_SEQ.swap(eid.seq_no, Ordering::Relaxed);
                }
            }
            Ok(())
        }
    }

    struct FakeGameLobbyRepo {
        pub contents: Arc<Mutex<GameLobby>>,
        pub put_in: Sender<GameLobby>,
    }

    impl GameLobbyRepo for FakeGameLobbyRepo {
        fn get(&self) -> Result<GameLobby, FetchErr> {
            Ok(self.contents.lock().expect("mutex lock").clone())
        }
        fn put(&self, game_lobby: GameLobby) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("lock");
            *data = game_lobby.clone();
            Ok(self.put_in.send(game_lobby).expect("send"))
        }
    }

    struct FakeXRead {
        sorted_data: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
    }
    impl XRead for FakeXRead {
        /// Be careful, this implementation assumes
        /// that the underlying data is pre-sorted
        fn xread_sorted(
            &self,
            entry_ids: AllEntryIds,
        ) -> Result<Vec<(redis_streams::XReadEntryId, super::StreamInput)>, XReadErr> {
            {
                let data: Vec<_> = self
                    .sorted_data
                    .lock()
                    .expect("lock")
                    .iter()
                    .filter(|(eid, stream_data)| match stream_data {
                        StreamInput::CG(_) => entry_ids.create_game < *eid,
                        StreamInput::FPG(_) => entry_ids.find_public_game < *eid,
                        StreamInput::JPG(_) => entry_ids.join_private_game < *eid,
                        StreamInput::SD(_) => entry_ids.session_disconnected < *eid,
                    })
                    .cloned()
                    .collect();
                if data.is_empty() {
                    // stop the test thread from spinning like crazy
                    std::thread::sleep(Duration::from_millis(20))
                }
                Ok(data)
            }
        }
    }
    struct FakeXAdd(Sender<StreamOutput>);
    impl XAdd for FakeXAdd {
        fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr> {
            Ok(self.0.send(data).expect("send"))
        }
    }
    #[test]
    fn test_process() {
        let (eid_call_in, eid_call_out) = unbounded();
        let (xadd_call_in, xadd_call_out) = unbounded();
        let (put_game_lobby_in, put_game_lobby_out) = unbounded();

        let sorted_fake_stream = Arc::new(Mutex::new(vec![]));

        // set up a loop to process game lobby requests
        let fake_game_lobby_contents = Arc::new(Mutex::new(GameLobby::default()));
        let fgl = fake_game_lobby_contents.clone();
        std::thread::spawn(move || loop {
            select! {
                recv(put_game_lobby_out) -> msg => match msg {
                    Ok(GameLobby { games }) => *fgl.lock().expect("mutex lock") = GameLobby { games },
                    Err(_) => panic!("fail")
                }
            }
        });

        let sfs = sorted_fake_stream.clone();
        let fgl = fake_game_lobby_contents.clone();
        thread::spawn(move || {
            let components = Components {
                entry_id_repo: Box::new(FakeEIDRepo {
                    call_in: eid_call_in,
                }),
                game_lobby_repo: Box::new(FakeGameLobbyRepo {
                    contents: fgl,
                    put_in: put_game_lobby_in,
                }),
                xread: Box::new(FakeXRead {
                    sorted_data: sfs.clone(),
                }),
                xadd: Box::new(FakeXAdd(xadd_call_in)),
            };
            process(&components);
        });

        let timeout = Duration::from_millis(166);
        // assert that fetch_all is being called faithfully
        select! {
            recv(eid_call_out) -> msg => match msg {
                Ok(EIDRepoCalled::Fetch) => assert!(true),
                Ok(_) => panic!("out of order"),
                Err(_) => panic!("eid repo should fetch"),
            },
            default(timeout) => panic!("EID fetch time out")
        }

        // emit some events in a time-ordered fashion
        // (we need to use time-ordered push since the
        //   FakeXRead impl won't sort its underlying data )

        let mut fake_time_ms = 100;
        let incr_ms = 100;

        let session_b = SessionId::new();
        let session_w = SessionId::new();
        let client_b = ClientId::new();
        let client_w = ClientId::new();
        sorted_fake_stream.lock().expect("lock").push((
            quick_eid(fake_time_ms),
            StreamInput::FPG(FindPublicGame {
                client_id: client_w.clone(),
                session_id: session_w.clone(),
            }),
        ));

        thread::sleep(timeout);
        // The game lobby repo should now contain one game
        assert_eq!(
            fake_game_lobby_contents
                .clone()
                .lock()
                .expect("gl")
                .games
                .iter()
                .collect::<Vec<_>>()
                .len(),
            1
        );

        // There should be an XADD triggered for a wait-for-opponent
        // message
        select! {
            recv(xadd_call_out) -> msg => match msg {
                Ok(StreamOutput::WFO(_)) => assert!(true),
                _ => panic!("wrong output")
            },
            default(timeout) => panic!("WAIT timeout")
        }

        loop {
            // The EID repo record for Find Public Game
            // should have been advanced
            select! {
                recv(eid_call_out) -> msg => match msg {
                    Ok(EIDRepoCalled::Update(EntryIdType::FindPublicGameCmd, eid)) => {
                        assert_eq!(eid.millis_time, fake_time_ms);
                        break
                    },
                    Ok(_) => continue,
                    Err(_) => panic!("eid repo should update"),
                },
                default(timeout) => panic!("EID time out")
            }
        }

        fake_time_ms += incr_ms;
        sorted_fake_stream.lock().expect("lock").push((
            quick_eid(fake_time_ms),
            StreamInput::FPG(FindPublicGame {
                client_id: client_b,
                session_id: session_b,
            }),
        ));

        // There should now be GameReady in stream
        select! {
            recv(xadd_call_out) -> msg => match msg {
                Ok(StreamOutput::GR(_)) => assert!(true),
                _ => assert!(false)
            },
            default(timeout) => panic!("GR time out")
        }
    }

    fn quick_eid(ms: u64) -> XReadEntryId {
        XReadEntryId {
            millis_time: ms,
            seq_no: 0,
        }
    }
}
