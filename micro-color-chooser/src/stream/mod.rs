mod init;
mod topics;
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

const GROUP_NAME: &str = "micro-color-chooser";

pub fn process(components: &Components) {
    let mut gr_processed: Vec<XReadEntryId> = vec![];
    let mut ccp_processed: Vec<XReadEntryId> = vec![];
    loop {
        match components.xread.sorted() {
            Ok(_) => todo!(),
            Err(_) => error!("xread"),
        }

        if !gr_processed.is_empty() {
            if let Err(_e) = components.xread.ack_game_ready(&gr_processed) {
                error!("ack for game states failed")
            } else {
                gr_processed.clear()
            }
        }
        if !ccp_processed.is_empty() {
            if let Err(_e) = components.xread.ack_choose_color_pref(&ccp_processed) {
                ccp_processed.clear()
            }
        }
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
        gr_ack_ms: Arc<AtomicU64>,
        ccp_ack_ms: Arc<AtomicU64>,
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

    impl XRead for FakeXRead {
        fn sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr> {
            todo!()
        }

        fn ack_choose_color_pref(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            if let Some(max_id_millis) = ids.iter().map(|id| id.millis_time).max() {
                self.ccp_ack_ms.swap(max_id_millis, Relaxed);
            }

            Ok(())
        }

        fn ack_game_ready(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            if let Some(max_id_millis) = ids.iter().map(|id| id.millis_time).max() {
                self.gr_ack_ms.swap(max_id_millis, Relaxed);
            }

            Ok(())
        }
    }

    impl XAdd for FakeXAdd {
        fn xadd(&self, data: ColorsChosen) -> Result<(), XAddErr> {
            todo!()
        }
    }

    fn quick_eid(millis_time: u64) -> XReadEntryId {
        XReadEntryId {
            millis_time,
            seq_no: 0,
        }
    }

    struct TestOutputs {
        pub xadd_call_out: Receiver<ColorsChosen>,
        pub put_prefs_out: Receiver<SessionColorPref>,
        pub put_session_game_out: Receiver<SessionGame>,
        pub prefs_contents: Arc<Mutex<HashMap<GameId, GameColorPref>>>,
        pub session_game_contents: Arc<Mutex<HashMap<SessionId, SessionGame>>>,
    }

    fn run(
        first_color_pref: &ChooseColorPref,
        second_color_pref: &ChooseColorPref,
        game_id: &GameId,
    ) -> TestOutputs {
        let gr_ack_ms = Arc::new(AtomicU64::new(0));
        let ccp_ack_ms = Arc::new(AtomicU64::new(0));

        let (xadd_call_in, xadd_call_out): (_, Receiver<ColorsChosen>) = unbounded();
        let (put_prefs_in, put_prefs_out): (_, Receiver<SessionColorPref>) = unbounded();
        let (put_session_game_in, put_session_game_out): (_, Receiver<SessionGame>) = unbounded();

        let fake_prefs_contents: Arc<Mutex<HashMap<GameId, GameColorPref>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let fake_session_game_contents: Arc<Mutex<HashMap<SessionId, SessionGame>>> =
            Arc::new(Mutex::new(HashMap::new()));

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
        let fsg = fake_session_game_contents.clone();
        let ca = ccp_ack_ms.clone();
        let gra = gr_ack_ms.clone();
        thread::spawn(move || {
            let components = Components {
                session_game_repo: Box::new(FakeGameRepo {
                    contents: fsg,
                    put_in: put_session_game_in,
                }),
                prefs_repo: Box::new(FakePrefsRepo {
                    contents: fp,
                    put_in: put_prefs_in,
                }),
                xread: Box::new(FakeXRead {
                    gr_ack_ms: gra,
                    ccp_ack_ms: ca,
                    sorted_data: sfs.clone(),
                }),
                xadd: Box::new(FakeXAdd(xadd_call_in)),
            };
            process(&components);
        });

        // emit some events in a time-ordered fashion
        // (fake xread impl expects time ordering 😁)

        let wait_time = Duration::from_millis(166);
        let mut fake_time_ms = 100;
        let incr_ms = 100;

        let first_pref_xid = quick_eid(fake_time_ms);

        sorted_fake_stream
            .lock()
            .expect("lock")
            .push((first_pref_xid, StreamInput::CCP(first_color_pref.clone())));

        fake_time_ms += incr_ms;
        thread::sleep(wait_time);

        assert_eq!(ccp_ack_ms.load(Relaxed), first_pref_xid.millis_time);

        let second_pref_xid = quick_eid(fake_time_ms);

        sorted_fake_stream
            .lock()
            .expect("lock")
            .push((second_pref_xid, StreamInput::CCP(second_color_pref.clone())));

        fake_time_ms += incr_ms;
        thread::sleep(wait_time);

        let game_ready = GameReady {
            game_id: game_id.clone(),
            sessions: (
                first_color_pref.session_id.clone(),
                second_color_pref.session_id.clone(),
            ),
            event_id: EventId::new(),
        };

        let game_ready_xid = quick_eid(fake_time_ms);

        sorted_fake_stream
            .lock()
            .expect("lock")
            .push((game_ready_xid, StreamInput::GR(game_ready)));

        // check ack for game_states stream
        let found_gr_ack_ms = gr_ack_ms.load(Relaxed);
        let found_ccp_ack_ms = ccp_ack_ms.load(Relaxed);
        assert_eq!(found_gr_ack_ms, game_ready_xid.millis_time);
        assert_eq!(found_ccp_ack_ms, second_pref_xid.millis_time);
        TestOutputs {
            xadd_call_out,
            put_prefs_out,
            put_session_game_out,
            prefs_contents: fake_prefs_contents,
            session_game_contents: fake_session_game_contents,
        }
    }

    #[test]
    fn test_no_conflict() {
        let game_id = GameId(Uuid::new_v4());
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));

        let first_client_pref = ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0,
            color_pref: ColorPref::White,
        };
        let second_client_pref = ChooseColorPref {
            client_id: clients.1,
            session_id: sessions.1,
            color_pref: ColorPref::Black,
        };
        let test_outputs = run(&first_client_pref, &second_client_pref, &game_id);
        todo!("write test")
    }
}
