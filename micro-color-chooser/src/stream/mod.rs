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
use crate::service::{choose, game_color_prefs};

use log::error;
use redis_streams::XReadEntryId;

#[derive(Clone)]
pub enum StreamInput {
    GR(GameReady),
    CCP(ChooseColorPref),
}

const GROUP_NAME: &str = "micro-color-chooser";

pub fn process(components: &Components) {
    let repos = Repos::from(components);
    loop {
        let mut gr_processed: Vec<XReadEntryId> = vec![];
        let mut ccp_processed: Vec<XReadEntryId> = vec![];
        match components.xread.sorted() {
            Ok(xrr) => {
                for time_ordered_event in xrr {
                    match time_ordered_event {
                        (
                            xid,
                            StreamInput::CCP(ChooseColorPref {
                                client_id: _,
                                color_pref,
                                session_id,
                            }),
                        ) => {
                            let scp = SessionColorPref {
                                color_pref,
                                session_id: session_id.clone(),
                            };

                            if let Err(_e) = components.prefs_repo.put(&scp) {
                                error!("write to pref repo")
                            }

                            match game_color_prefs::by_session_id(&session_id, &repos) {
                                Ok(GameColorPref::Complete { game_id, prefs }) => {
                                    let colors_chosen = choose(&prefs.0, &prefs.1);
                                    if let Err(_e) = components.xadd.xadd(colors_chosen) {
                                        error!("error writing to colors chose stream")
                                    }
                                }
                                Ok(_) => {
                                    // do nothing until we know what both sides prefer
                                }
                                Err(_e) => {
                                    error!("fetch error checking game color prefs by session ID")
                                }
                            }

                            ccp_processed.push(xid)
                        }
                        (xid, StreamInput::GR(gr)) => {
                            if let Err(_e) = components.session_game_repo.put(SessionGame {
                                session_id: gr.sessions.0.clone(),
                                game_id: gr.game_id.clone(),
                            }) {
                                error!("write to session game repo 0")
                            }
                            if let Err(_e) = components.session_game_repo.put(SessionGame {
                                session_id: gr.sessions.1.clone(),
                                game_id: gr.game_id.clone(),
                            }) {
                                error!("write to session game repo 0")
                            }

                            match game_color_prefs::by_game_ready(&gr, &repos) {
                                Ok(GameColorPref::Complete { game_id, prefs }) => {
                                    let colors_chosen = choose(&prefs.0, &prefs.1);
                                    if let Err(_e) = components.xadd.xadd(colors_chosen) {
                                        error!("error writing to colors chose stream")
                                    }
                                }
                                Ok(_) => {
                                    // do nothing until we know what both sides prefer
                                }
                                Err(_e) => {
                                    error!("fetch error checking game color prefs by session ID")
                                }
                            }

                            gr_processed.push(xid)
                        }
                    }
                }
            }
            Err(_) => error!("xread"),
        }

        if !gr_processed.is_empty() {
            if let Err(_e) = components.xread.ack_game_ready(&gr_processed) {
                error!("ack for game ready failed")
            }
        }
        if !ccp_processed.is_empty() {
            if let Err(_e) = components.xread.ack_choose_color_pref(&ccp_processed) {
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
    use crossbeam_channel::{select, unbounded, Receiver, Sender};
    use redis_streams::XReadEntryId;
    use std::collections::HashMap;
    use std::rc::Rc;
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
        pub contents: Arc<Mutex<HashMap<SessionId, SessionColorPref>>>,
        pub put_in: Sender<SessionColorPref>,
    }

    struct FakeXAdd(Sender<ColorsChosen>);
    struct FakeXRead {
        gr_ack_ms: Arc<AtomicU64>,
        ccp_ack_ms: Arc<AtomicU64>,
        max_read_millis: AtomicU64,
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
        pub prefs_contents: Arc<Mutex<HashMap<SessionId, SessionColorPref>>>,
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

        let fake_prefs_contents: Arc<Mutex<HashMap<SessionId, SessionColorPref>>> =
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
                session_game_repo: Rc::new(FakeGameRepo {
                    contents: fsg,
                    put_in: put_session_game_in,
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

        assert_eq!(found_ccp_ack_ms, second_pref_xid.millis_time);

        assert_eq!(found_gr_ack_ms, game_ready_xid.millis_time);

        TestOutputs {
            xadd_call_out,
            put_prefs_out,
            put_session_game_out,
            prefs_contents: fake_prefs_contents,
            session_game_contents: fake_session_game_contents,
        }
    }

    #[test]
    fn happy_path() {
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
