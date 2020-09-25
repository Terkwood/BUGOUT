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
    use crate::api::*;
    use crate::repo::*;
    use crate::Components;
    use crossbeam_channel::{select, unbounded, Receiver, Sender};
    use redis_streams::XReadEntryId;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    struct FakeGameRepo {
        pub contents: Arc<Mutex<HashMap<SessionId, SessionGame>>>,
        pub put_in: Sender<SessionGame>,
    }
    struct FakePrefsRepo {
        pub contents: Arc<Mutex<HashMap<GameId, GameColorPref>>>,
        pub put_in: Sender<SessionColorPref>,
    }

    struct FakeXAdd(Sender<ColorsChosen>);
    struct FakeXRead {
        sorted_data: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
    }

    impl SessionGameRepo for FakeGameRepo {
        fn get(&self, session_id: &SessionId) -> Result<Option<SessionGame>, FetchErr> {
            Ok(self
                .contents
                .lock()
                .expect("mutex")
                .get(session_id)
                .map(|g| g.clone()))
        }

        fn put(&self, session_game: SessionGame) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            data.insert(session_game.session_id.clone(), session_game.clone());
            Ok(self.put_in.send(session_game).expect("send"))
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
            Ok(self.put_in.send(scp).expect("send"))
        }
    }

    impl XRead for FakeXRead {}

    impl XAdd for FakeXAdd {
        fn xadd(&self, data: ColorsChosen) -> Result<(), XAddErr> {
            todo!()
        }
    }

    struct TestOutputs {
        pub xadd_call_out: Receiver<ColorsChosen>,
        pub put_prefs_out: Receiver<SessionColorPref>,
        pub put_session_game_out: Receiver<SessionGame>,
    }

    fn run(c1: &ChooseColorPref, c2: &ChooseColorPref, game_id: &GameId) -> TestOutputs {
        static GR_ACK_XID: AtomicU64 = AtomicU64::new(0);
        static CCP_ACK_XID: AtomicU64 = AtomicU64::new(0);

        let (xadd_call_in, xadd_call_out): (_, Receiver<ColorsChosen>) = unbounded();
        let (put_prefs_in, put_prefs_out): (_, Receiver<SessionColorPref>) = unbounded();
        let (put_session_game_in, put_session_game_out): (_, Receiver<SessionGame>) = unbounded();

        let fake_prefs_contents = Arc::new(Mutex::new(HashMap::new()));
        let fake_session_game_contents = Arc::new(Mutex::new(HashMap::new()));

        let sorted_fake_stream: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>> =
            Arc::new(Mutex::new(vec![]));

        // TODO
        // TODO
        // TODO
        // TODO
        // TODO check the test impl of micro-history-provider
        // TODO and trim the worthless put_in loops ?!
        // TODO
        // TODO
        // TODO
        // TODO

        let sfs = sorted_fake_stream.clone();
        let fp = fake_prefs_contents.clone();
        thread::spawn(move || {
            let components = Components {
                session_game_repo: Box::new(FakeGameRepo {
                    contents: fake_session_game_contents,
                    put_in: put_session_game_in,
                }),
                prefs_repo: Box::new(FakePrefsRepo {
                    contents: fp,
                    put_in: put_prefs_in,
                }),
                xread: Box::new(FakeXRead {
                    sorted_data: sfs.clone(),
                }),
                xadd: Box::new(FakeXAdd(xadd_call_in)),
            };
            process(&components);
        });

        todo!();

        TestOutputs {
            xadd_call_out,
            put_prefs_out,
            put_session_game_out,
        }
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
