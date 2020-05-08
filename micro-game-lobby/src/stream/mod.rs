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
    use crate::components::Components;
    use crate::repo::*;
    struct FakeRedis;
    impl EntryIdRepo for FakeRedis {
        fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
            unimplemented!()
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
    #[test]
    fn test_process() {
        let components = Components {
            entry_id_repo: Box::new(FakeRedis),
            game_lobby_repo: Box::new(FakeRedis),
            xreader: todo!(),
        };
        todo!("write a unit test")
    }
}
