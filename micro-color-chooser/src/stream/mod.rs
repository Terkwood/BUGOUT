mod init;
mod topics;
mod xadd;
mod xread;

pub use init::*;
pub use xadd::*;
pub use xread::*;

use crate::components::*;
use crate::service::{choose, game_color_prefs};
use color_model::api::*;
use color_model::*;

use log::error;
use redis_streams::XReadEntryId;

#[derive(Clone)]
pub enum StreamInput {
    GR(GameReady),
    CCP(ChooseColorPref),
}

enum Processed {
    GR(XReadEntryId),
    CCP(XReadEntryId),
}

const GROUP_NAME: &str = "micro-color-chooser";

const ACK_QUEUE_CAPACITY: usize = 25;

pub fn process(components: &mut Components) {
    let repos = Repos::new(components);
    loop {
        let mut gr_processed: Vec<XReadEntryId> = Vec::with_capacity(ACK_QUEUE_CAPACITY);
        let mut ccp_processed: Vec<XReadEntryId> = Vec::with_capacity(ACK_QUEUE_CAPACITY);
        match components.xread.sorted() {
            Ok(xrr) => {
                for time_ordered_event in xrr {
                    let (result, pxid) = match time_ordered_event {
                        (
                            xid,
                            StreamInput::CCP(ChooseColorPref {
                                client_id,
                                color_pref,
                                session_id,
                            }),
                        ) => {
                            let scp = SessionColorPref {
                                color_pref,
                                session_id: session_id.clone(),
                                client_id,
                            };

                            if let Err(_e) = components.prefs_repo.put(&scp) {
                                error!("write to pref repo")
                            }

                            (
                                game_color_prefs::by_session_id(&session_id, &repos),
                                Processed::CCP(xid),
                            )
                        }
                        (xid, StreamInput::GR(gr)) => {
                            if let Err(_e) = components.game_ready_repo.put(gr.clone()) {
                                error!("write to session game repo 0")
                            }
                            if let Err(_e) = components.game_ready_repo.put(gr.clone()) {
                                error!("write to session game repo 0")
                            }

                            (
                                game_color_prefs::by_game_ready(&gr, &repos),
                                Processed::GR(xid),
                            )
                        }
                    };

                    match result {
                        Ok(GameColorPref::Complete { game_id, prefs }) => {
                            let colors_chosen =
                                choose(&prefs.0, &prefs.1, &game_id, &mut components.random);
                            if let Err(_e) = components.xadd.xadd(colors_chosen) {
                                error!("error writing to colors chose stream")
                            }
                        }
                        Ok(_) => {
                            // do nothing until we know what both sides prefer
                        }
                        Err(_e) => error!("fetch error checking game color prefs by session ID"),
                    }

                    match pxid {
                        Processed::CCP(xid) => ccp_processed.push(xid),
                        Processed::GR(xid) => gr_processed.push(xid),
                    }
                }
            }
            Err(_) => error!("xread"),
        }

        if !gr_processed.is_empty() {
            if let Err(_e) = &components.xread.ack_game_ready(&gr_processed) {
                error!("ack for game ready failed")
            }
        }
        if !ccp_processed.is_empty() {
            if let Err(_e) = &components.xread.ack_choose_color_pref(&ccp_processed) {
                error!("ack for choose color prefs failed")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::*;
    use crate::Components;
    use core_model::*;
    use crossbeam_channel::{unbounded, Receiver, Sender};
    use redis_streams::XReadEntryId;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    struct FakeGameRepo {
        pub contents: Arc<Mutex<HashMap<SessionId, GameReady>>>,
        pub put_in: Sender<GameReady>,
    }

    impl GameReadyRepo for FakeGameRepo {
        fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr> {
            Ok(self
                .contents
                .lock()
                .expect("mutex")
                .get(session_id)
                .map(|g| g.clone()))
        }

        fn put(&self, game_ready: GameReady) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            data.insert(game_ready.sessions.0.clone(), game_ready.clone());
            data.insert(game_ready.sessions.1.clone(), game_ready.clone());
            Ok(self.put_in.send(game_ready).expect("send"))
        }
    }

    struct FakePrefsRepo {
        pub contents: Arc<Mutex<HashMap<SessionId, SessionColorPref>>>,
        pub put_in: Sender<SessionColorPref>,
    }

    impl PrefsRepo for FakePrefsRepo {
        fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr> {
            Ok(self
                .contents
                .lock()
                .expect("mutex")
                .get(session_id)
                .map(|gcp| gcp.clone()))
        }

        fn put(&self, scp: &SessionColorPref) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            data.insert(scp.session_id.clone(), scp.clone());
            Ok(self.put_in.send(scp.clone()).expect("send"))
        }
    }

    struct FakeXRead {
        gr_ack_ms: Arc<AtomicU64>,
        ccp_ack_ms: Arc<AtomicU64>,
        max_read_millis: AtomicU64,
        sorted_data: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
    }

    impl XRead for FakeXRead {
        fn sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr> {
            let max_ms = self.max_read_millis.load(Relaxed);
            let data: Vec<_> = self
                .sorted_data
                .lock()
                .expect("lock")
                .iter()
                .filter(|(eid, stream_data)| match stream_data {
                    StreamInput::CCP(_) => max_ms < eid.millis_time,
                    StreamInput::GR(_) => max_ms < eid.millis_time,
                })
                .cloned()
                .collect();

            if data.is_empty() {
                // stop the test thread from spinning like crazy
                std::thread::sleep(Duration::from_millis(20))
            } else {
                // this hack is standing in for "xreadgroup >" semantics
                let new_max_eid_millis = data.iter().map(|(eid, _)| eid).max().unwrap();
                self.max_read_millis
                    .swap(new_max_eid_millis.millis_time, Relaxed);
            }
            Ok(data)
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

    struct FakeXAdd(Sender<ColorsChosen>);
    impl XAdd for FakeXAdd {
        fn xadd(&self, data: ColorsChosen) -> Result<(), XAddErr> {
            Ok(self.0.send(data).expect("send"))
        }
    }

    fn quick_xid(millis_time: u64) -> XReadEntryId {
        XReadEntryId {
            millis_time,
            seq_no: 0,
        }
    }

    struct TestOutputs {
        pub xadd_call_out: Receiver<ColorsChosen>,
        pub put_prefs_out: Receiver<SessionColorPref>,
        pub put_game_ready_out: Receiver<GameReady>,
        pub prefs_contents: Arc<Mutex<HashMap<SessionId, SessionColorPref>>>,
        pub game_ready_contents: Arc<Mutex<HashMap<SessionId, GameReady>>>,
    }

    fn run_stream(events: Vec<StreamInput>) -> TestOutputs {
        let gr_ack_ms = Arc::new(AtomicU64::new(0));
        let ccp_ack_ms = Arc::new(AtomicU64::new(0));

        let (xadd_call_in, xadd_call_out): (_, Receiver<ColorsChosen>) = unbounded();
        let (put_prefs_in, put_prefs_out): (_, Receiver<SessionColorPref>) = unbounded();
        let (put_game_ready_in, put_game_ready_out): (_, Receiver<GameReady>) = unbounded();

        let fake_prefs_contents: Arc<Mutex<HashMap<SessionId, SessionColorPref>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let fake_game_ready_contents: Arc<Mutex<HashMap<SessionId, GameReady>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let sorted_fake_stream: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>> =
            Arc::new(Mutex::new(vec![]));

        let sfs = sorted_fake_stream.clone();
        let fp = fake_prefs_contents.clone();
        let fsg = fake_game_ready_contents.clone();
        let ca = ccp_ack_ms.clone();
        let gra = gr_ack_ms.clone();
        thread::spawn(move || {
            let mut components = Components {
                game_ready_repo: Rc::new(FakeGameRepo {
                    contents: fsg,
                    put_in: put_game_ready_in,
                }),
                prefs_repo: Rc::new(FakePrefsRepo {
                    contents: fp,
                    put_in: put_prefs_in,
                }),
                xread: Box::new(FakeXRead {
                    gr_ack_ms: gra,
                    ccp_ack_ms: ca,
                    sorted_data: sfs.clone(),
                    max_read_millis: AtomicU64::new(0),
                }),
                xadd: Box::new(FakeXAdd(xadd_call_in)),
                random: crate::service::Random::new(),
            };
            process(&mut components);
        });

        // emit some events in a time-ordered fashion
        // (fake xread impl expects time ordering 😁)

        let wait_time = Duration::from_millis(166);
        let mut fake_time_ms = 100;
        let incr_ms = 100;

        for event in events {
            let xid = quick_xid(fake_time_ms);

            sorted_fake_stream
                .lock()
                .expect("lock")
                .push((xid, event.clone()));

            fake_time_ms += incr_ms;
            thread::sleep(wait_time);

            match event {
                StreamInput::CCP(_) => assert_eq!(ccp_ack_ms.load(Relaxed), xid.millis_time),
                StreamInput::GR(_) => assert_eq!(gr_ack_ms.load(Relaxed), xid.millis_time),
            }
        }

        TestOutputs {
            xadd_call_out,
            put_prefs_out,
            put_game_ready_out,
            prefs_contents: fake_prefs_contents,
            game_ready_contents: fake_game_ready_contents,
        }
    }

    #[test]
    fn happy_path_1() {
        let game_id = GameId(Uuid::new_v4());
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));

        let first_client_pref = StreamInput::CCP(ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0.clone(),
            color_pref: ColorPref::White,
        });
        let second_client_pref = StreamInput::CCP(ChooseColorPref {
            client_id: clients.1,
            session_id: sessions.1.clone(),
            color_pref: ColorPref::Black,
        });

        let board_size = 9;
        let game_ready = StreamInput::GR(GameReady {
            game_id,
            sessions,
            event_id: EventId::new(),
            board_size,
        });
        let test_outputs = run_stream(vec![first_client_pref, second_client_pref, game_ready]);

        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_game_ready_out.recv().expect("recv");
        test_outputs.xadd_call_out.recv().expect("recv");
    }

    #[test]
    fn happy_path_2() {
        let game_id = GameId(Uuid::new_v4());
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));

        let first_client_pref = StreamInput::CCP(ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0.clone(),
            color_pref: ColorPref::White,
        });
        let second_client_pref = StreamInput::CCP(ChooseColorPref {
            client_id: clients.1,
            session_id: sessions.1.clone(),
            color_pref: ColorPref::Black,
        });

        let board_size = 9;
        let game_ready = StreamInput::GR(GameReady {
            game_id,
            sessions,
            event_id: EventId::new(),
            board_size,
        });
        let test_outputs = run_stream(vec![game_ready, first_client_pref, second_client_pref]);

        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_game_ready_out.recv().expect("recv");
        test_outputs.xadd_call_out.recv().expect("recv");
    }

    #[test]
    fn no_game_ready_no_choice() {
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));

        let first_client_pref = StreamInput::CCP(ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0.clone(),
            color_pref: ColorPref::White,
        });
        let second_client_pref = StreamInput::CCP(ChooseColorPref {
            client_id: clients.1,
            session_id: sessions.1.clone(),
            color_pref: ColorPref::Black,
        });

        let test_outputs = run_stream(vec![first_client_pref, second_client_pref]);

        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_prefs_out.recv().expect("recv");
        assert!(test_outputs.xadd_call_out.is_empty())
    }

    #[test]
    fn no_full_pref_no_choice() {
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));

        let first_client_pref = StreamInput::CCP(ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0.clone(),
            color_pref: ColorPref::White,
        });
        let game_id = GameId::new();
        let board_size = 9;
        let game_ready = StreamInput::GR(GameReady {
            game_id,
            sessions,
            event_id: EventId::new(),
            board_size,
        });

        let test_outputs = run_stream(vec![first_client_pref, game_ready]);

        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_game_ready_out.recv().expect("recv");
        assert!(test_outputs.xadd_call_out.is_empty())
    }
}
