mod init;
mod xack;
mod xadd;
mod xread;

pub use init::*;
pub use xack::*;
pub use xadd::*;
pub use xread::*;

use crate::components::Components;
use crate::game_lobby::GameLobbyOps;
use crate::PUBLIC_GAME_BOARD_SIZE;
use core_model::*;
use lobby_model::api::*;
use lobby_model::*;
use log::{error, info, warn};
use redis_streams::XReadEntryId;

pub const GROUP_NAME: &str = "micro-game-lobby";

pub fn process(reg: &Components) {
    loop {
        match reg.xread.xread_sorted() {
            Ok(xrr) => {
                let mut to_ack = vec![];
                for (xid, data) in xrr {
                    info!("🧮 Processing {:?}", &data);
                    consume(xid, &data, &reg);
                    info!("🛎 OK {:?}", &data);
                    to_ack.push((xid, data));
                }
            }
            Err(e) => error!("Stream err {:?}", e),
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
    let visibility = Visibility::Public;
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
                visibility,
                game_id: game_id.clone(),
            })) {
                error!("game lobby write F2");
            } else {
                if let Err(_) = reg.xadd.xadd(StreamOutput::WFO(WaitForOpponent {
                    event_id: EventId::new(),
                    game_id,
                    session_id: session_id.clone(),
                    visibility,
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
                visibility: cg.visibility,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::components::Components;
    use crate::repo::*;
    use crossbeam_channel::{select, unbounded, Sender};
    use redis_streams::XReadEntryId;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use std::thread;

    struct FakeGameLobbyRepo {
        pub contents: Arc<Mutex<GameLobby>>,
    }

    impl GameLobbyRepo for FakeGameLobbyRepo {
        fn get(&self) -> Result<GameLobby, FetchErr> {
            Ok(self.contents.lock().expect("mutex lock").clone())
        }
        fn put(&self, game_lobby: GameLobby) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("lock");
            *data = game_lobby.clone();
            Ok(())
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
        ) -> Result<Vec<(redis_streams::XReadEntryId, super::StreamInput)>, XReadErr> {
            {
                let mut data: Vec<_> = self.sorted_data.lock().expect("lock").to_vec();
                if data.is_empty() {
                    // stop the test thread from spinning like crazy
                    std::thread::sleep(Duration::from_millis(20));
                    Ok(vec![])
                } else {
                    let result = data.clone();
                    data.clear();
                    Ok(result)
                }
            }
        }
    }
    struct FakeXAdd(Sender<StreamOutput>);
    impl XAdd for FakeXAdd {
        fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr> {
            Ok(self.0.send(data).unwrap_or_default())
        }
    }
    #[test]
    fn test_process() {
        let (xadd_call_in, xadd_call_out) = unbounded();

        let sorted_fake_stream = Arc::new(Mutex::new(vec![]));

        let timeout = Duration::from_millis(160);

        // set up a loop to process game lobby requests
        let fake_game_lobby_contents = Arc::new(Mutex::new(GameLobby::default()));

        let sfs = sorted_fake_stream.clone();
        let fgl = fake_game_lobby_contents.clone();

        thread::spawn(move || {
            let components = Components {
                game_lobby_repo: Box::new(FakeGameLobbyRepo { contents: fgl }),
                xread: Box::new(FakeXRead {
                    sorted_data: sfs.clone(),
                }),
                xadd: Box::new(FakeXAdd(xadd_call_in)),
                xack: todo!(),
            };
            process(&components);
        });

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
