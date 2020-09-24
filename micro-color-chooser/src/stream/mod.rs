mod init;
mod xadd;
mod xread;

pub use init::*;
pub use xadd::*;
pub use xread::*;

use crate::api::*;
use crate::components::*;
use crate::model::*;
use log::{error, warn};
use redis::Commands;
use redis_streams::XReadEntryId;

pub enum StreamInput {
    GR(GameReady),
    CCP(ChooseColorPref),
}

pub fn process(components: &Components) {
    todo!("ack id arrays");
    let mut gs_processed: Vec<XReadEntryId> = vec![];
    loop {
        todo!("match components.xread.xread_sorted()");
        /*Ok(_) => {
            for time_ordered_event in todo!("records") {
                todo!("match time_ordered_event")
            }
        }
        Err(_) => error!("xread"),*/

        todo!("acks");
        /*if !gs_processed.is_empty() {
            if let Err(_e) = components.xread.xack_game_states(&gs_processed) {
                error!("ack for game states failed")
            }
        }*/
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::*;
    use crate::Components;
    use crossbeam_channel::{select, unbounded, Sender};
    use redis_streams::XReadEntryId;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    struct FakeGameRepo {
        pub contents: Arc<Mutex<HashMap<SessionId, GameId>>>,
        pub put_in: Sender<GameId>,
    }
    struct FakePrefsRepo {
        pub contents: Arc<Mutex<HashMap<GameId, GameColorPref>>>,
        pub put_in: Sender<GameColorPref>,
    }

    impl GameRepo for FakeGameRepo {
        fn get(&self, session_id: &SessionId) -> Result<Option<GameId>, FetchErr> {
            Ok(self
                .contents
                .lock()
                .expect("mutex")
                .get(session_id)
                .map(|g| g.clone()))
        }

        fn put(&self, session_id: &SessionId, game_id: &GameId) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            data.insert(session_id.clone(), game_id.clone());
            Ok(self.put_in.send(game_id.clone()).expect("send"))
        }
    }

    impl PrefsRepo for FakePrefsRepo {
        fn get(&self, game_id: &GameId) -> Result<GameColorPref, FetchErr> {
            Ok(self
                .contents
                .lock()
                .expect("mutex")
                .get(game_id)
                .map(|gcp| gcp.clone())
                .unwrap_or(GameColorPref::Empty))
        }

        fn add(&self, scp: SessionColorPref) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            match data.get(&scp.game_id).cloned() {
                None => {
                    data.insert(scp.game_id.clone(), GameColorPref::Partial(scp.clone()));
                }
                Some(GameColorPref::Partial(first)) => {
                    data.insert(
                        scp.game_id.clone(),
                        GameColorPref::Complete(first.clone(), scp.clone()),
                    );
                }
                Some(_) => panic!("prefs already complete"),
            }
            Ok(())
        }
    }

    fn run(c1: &ChooseColorPref, c2: &ChooseColorPref, game_id: &GameId) {
        static GR_ACK_XID: AtomicU64 = AtomicU64::new(0);
        static CCP_ACK_XID: AtomicU64 = AtomicU64::new(0);
    }

    #[test]
    fn test_no_conflict() {
        let game_id = GameId(Uuid::new_v4());
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));
        let game_ready_event = GameReady {
            game_id,
            sessions,
            event_id: EventId::new(),
        };
        todo!("write test");
    }
}
