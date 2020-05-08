mod xread;

pub use xread::*;

use crate::components::Components;
use crate::*;
use log::error;

pub fn process(topics: &topics::StreamTopics, components: &Components) {
    loop {
        match components.entry_id_repo.fetch_all() {
            Ok(all_eids) => match components.xreader.xread_sorted(all_eids, topics) {
                Ok(xrr) => {
                    for time_ordered_event in xrr {
                        match time_ordered_event {
                            (_eid, StreamData::FPG(_)) => todo!(),
                            (_eid, StreamData::CG(_)) => todo!(),
                            (_eid, StreamData::JPG(_)) => todo!(),
                        }
                    }
                }
                Err(e) => error!("Stream err {}", e),
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
    use crate::topics::StreamTopics;
    use crossbeam_channel::{select, Receiver, Sender};
    use std::thread;
    struct FakeRedis;
    impl EntryIdRepo for FakeRedis {
        fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
            unimplemented!()
        }
        fn update(
            &self,
            _eid_type: EntryIdType,
            _eid: redis_streams::XReadEntryId,
        ) -> Result<(), WriteErr> {
            todo!()
        }
    }
    impl GameLobbyRepo for FakeRedis {
        fn get(&self) -> Result<crate::game_lobby::GameLobby, FetchErr> {
            todo!()
        }
        fn put(&self, game_lobby: crate::game_lobby::GameLobby) -> Result<(), WriteErr> {
            todo!()
        }
    }
    impl XReader for FakeRedis {
        fn xread_sorted(
            &self,
            _entry_ids: AllEntryIds,
            _topics: &crate::topics::StreamTopics,
        ) -> Result<
            Vec<(redis_streams::XReadEntryId, super::StreamData)>,
            redis_conn_pool::redis::RedisError,
        > {
            todo!()
        }
    }
    #[test]
    fn test_process() {
        thread::spawn(|| {
            let components = Components {
                entry_id_repo: Box::new(FakeRedis),
                game_lobby_repo: Box::new(FakeRedis),
                xreader: Box::new(FakeRedis),
            };
            process(&StreamTopics::default(), &components);
        });
    }
}
