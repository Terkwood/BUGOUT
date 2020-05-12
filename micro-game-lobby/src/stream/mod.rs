mod xadd;
mod xread;

pub use xadd::*;
pub use xread::*;

use crate::components::Components;
use crate::repo::EntryIdType;
use crate::*;
use log::error;
use redis_streams::XReadEntryId;

pub fn process(components: &Components) {
    loop {
        match components.entry_id_repo.fetch_all() {
            Ok(all_eids) => match components.xread.xread_sorted(all_eids) {
                Ok(xrr) => {
                    for (eid, data) in xrr {
                        consume(eid, &data);
                        increment(eid, data, components);
                    }
                }
                Err(e) => error!("Stream err {:?}", e),
            },
            Err(e) => error!("Failed to fetch EIDs {:?}", e),
        }
    }
}

fn consume(_eid: XReadEntryId, _event: &StreamData) {}
fn increment(eid: XReadEntryId, event: StreamData, components: &Components) {
    if let Err(e) = components
        .entry_id_repo
        .update(EntryIdType::from(event), eid)
    {
        error!("eid write {:?}", e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::components::Components;
    use crate::game_lobby::*;
    use crate::repo::*;
    use crossbeam_channel::{select, unbounded, Sender};
    use redis_streams::XReadEntryId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};

    use std::thread;
    static FAKE_FPG_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_CG_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_JPG_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_FPG_SEQ: AtomicU64 = AtomicU64::new(0);
    static FAKE_CG_SEQ: AtomicU64 = AtomicU64::new(0);
    static FAKE_JPG_SEQ: AtomicU64 = AtomicU64::new(0);
    struct FakeEIDRepo {
        call_in: Sender<EIDRepoCalled>,
    }
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
            Ok(self.put_in.send(game_lobby).expect("send"))
        }
    }
    struct FakeXRead;
    impl XRead for FakeXRead {
        fn xread_sorted(
            &self,
            _entry_ids: AllEntryIds,
        ) -> Result<Vec<(redis_streams::XReadEntryId, super::StreamData)>, XReadErr> {
            todo!()
        }
    }
    struct FakeXAdd;
    impl XAdd for FakeXAdd {}
    #[test]
    fn test_process() {
        let (eid_call_in, eid_call_out) = unbounded();
        let (put_game_lobby_in, put_game_lobby_out) = unbounded();

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

        thread::spawn(move || {
            let components = Components {
                entry_id_repo: Box::new(FakeEIDRepo {
                    call_in: eid_call_in,
                }),
                game_lobby_repo: Box::new(FakeGameLobbyRepo {
                    contents: fake_game_lobby_contents,
                    put_in: put_game_lobby_in,
                }),
                xread: Box::new(FakeXRead {}),
                xadd: Box::new(FakeXAdd {}),
            };
            process(&components);
        });

        select! { recv(eid_call_out) -> msg => match msg {
            Ok(EIDRepoCalled::Fetch) => assert!(true),
            Ok(_) => panic!("out of order"),
            Err(_) => panic!("eid repo should fetch"),
        }}
    }
}
