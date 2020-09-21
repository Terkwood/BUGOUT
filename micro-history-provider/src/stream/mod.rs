mod topics;
mod xadd;
mod xread;

pub use xadd::XAdd;
pub use xread::XRead;

use crate::api::*;
use crate::components::*;
use crate::model::*;
use log::warn;
use redis::Commands;

#[derive(Clone, Debug)]
pub enum StreamData {
    PH(ProvideHistory),
    GS(GameId, GameState),
}

pub fn process(components: &Components) {
    loop {
        todo!()
    }
}

const GROUP_NAME: &str = "micro-history-provider";

pub fn create_consumer_group(client: &redis::Client) {
    let mut conn = client.get_connection().expect("group create conn");
    let mm: Result<(), _> = conn.xgroup_create_mkstream(topics::PROVIDE_HISTORY, GROUP_NAME, "$");
    if let Err(e) = mm {
        warn!(
            "Ignoring error creating {} consumer group (it probably exists already) {:?}",
            topics::PROVIDE_HISTORY,
            e
        );
    }
    let gs: Result<(), _> =
        conn.xgroup_create_mkstream(topics::GAME_STATES_CHANGELOG, GROUP_NAME, "$");
    if let Err(e) = gs {
        warn!(
            "Ignoring error creating {} consumer group (it probably exists already) {:?}",
            topics::GAME_STATES_CHANGELOG,
            e
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repo::*;
    use crate::Components;
    use crossbeam_channel::{Receiver, Sender};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    static FAKE_PROV_HIST_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_GAME_STATES_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_PROV_HIST_SEQ: AtomicU64 = AtomicU64::new(0);
    static FAKE_GAME_STATES_SEQ: AtomicU64 = AtomicU64::new(0);

    struct FakeHistoryRepo {
        pub contents: Arc<Mutex<Option<Vec<Move>>>>,
        pub put_in: Sender<Vec<Move>>,
    }

    impl HistoryRepo for FakeHistoryRepo {
        fn get(&self, game_id: GameId) -> Result<Option<Vec<Move>>, FetchErr> {
            Ok(self.contents.lock().expect("mutex").clone())
        }

        fn put(&self, game_id: GameId, moves: Vec<Move>) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            *data = Some(moves.clone());
            Ok(self.put_in.send(moves).expect("send"))
        }
    }
}
