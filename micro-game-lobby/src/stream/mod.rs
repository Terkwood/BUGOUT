mod xread;

pub use xread::*;

use crate::components::Components;
use crate::*;
use log::error;

pub fn process(components: &Components) {
    loop {
        match components.entry_id_repo.fetch_all() {
            Ok(all_eids) => match components.xreader.xread_sorted(all_eids) {
                Ok(xrr) => {
                    for time_ordered_event in xrr {
                        match time_ordered_event {
                            (_eid, StreamData::FPG(_)) => todo!(),
                            (_eid, StreamData::CG(_)) => todo!(),
                            (_eid, StreamData::JPG(_)) => todo!(),
                        }
                    }
                }
                Err(e) => error!("Stream err {:?}", e),
            },
            Err(e) => error!("Failed to fetch EIDs {:?}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::components::Components;
    use crate::repo::*;
    use crossbeam_channel::{select, unbounded, Receiver, Sender};
    use redis_streams::XReadEntryId;
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;
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

    struct FakeGameLobbyRepo;
    impl GameLobbyRepo for FakeGameLobbyRepo {
        fn get(&self) -> Result<crate::game_lobby::GameLobby, FetchErr> {
            todo!()
        }
        fn put(&self, _game_lobby: crate::game_lobby::GameLobby) -> Result<(), WriteErr> {
            todo!()
        }
    }
    struct FakeXReader;
    impl XReader for FakeXReader {
        fn xread_sorted(
            &self,
            _entry_ids: AllEntryIds,
        ) -> Result<Vec<(redis_streams::XReadEntryId, super::StreamData)>, XReadErr> {
            todo!()
        }
    }
    #[test]
    fn test_process() {
        let (eid_call_in, eid_call_out) = unbounded();

        thread::spawn(move || {
            let components = Components {
                entry_id_repo: Box::new(FakeEIDRepo {
                    call_in: eid_call_in,
                }),
                game_lobby_repo: Box::new(FakeGameLobbyRepo {}),
                xreader: Box::new(FakeXReader {}),
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
