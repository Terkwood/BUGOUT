mod init;
pub mod topics;
mod xadd;

pub use init::*;
pub use xadd::*;

use crate::components::*;
use crate::service::{choose, game_color_prefs};
use color_model::api::*;
use color_model::*;

use log::{error, info};
use redis_streams::{anyhow, Message, SortedStreams, XId};

const GROUP_NAME: &str = "micro-color-chooser";
use crate::service::Random;

use std::rc::Rc;
use std::sync::Mutex;
pub struct ColorChooserStreams {
    pub reg: Components,
    pub rng: Rc<Mutex<Random>>,
}

impl ColorChooserStreams {
    pub fn new(reg: Components) -> Self {
        let rng = Rc::new(Mutex::new(Random::new()));
        Self { reg, rng }
    }

    pub fn process(&self, streams: &mut dyn SortedStreams) {
        loop {
            if let Err(e) = streams.consume() {
                error!("Stream err {:?}", e)
            }
        }
    }

    pub fn consume_game_ready(&self, msg: &Message) -> anyhow::Result<()> {
        let maybe_value = msg.get("data");
        let repos = Repos::new(&self.reg);
        Ok(if let Some(redis::Value::Data(data)) = maybe_value {
            let gr: GameReady = bincode::deserialize(&data)?;

            if let Err(_e) = self.reg.game_ready_repo.put(gr.clone()) {
                error!("write to session game repo 0")
            }
            if let Err(_e) = self.reg.game_ready_repo.put(gr.clone()) {
                error!("write to session game repo 0")
            }

            let gcp: Result<GameColorPref, crate::repo::FetchErr> =
                game_color_prefs::by_game_ready(&gr, &repos);
            match gcp {
                Ok(GameColorPref::Complete { game_id, prefs }) => {
                    let mutex = self.rng.lock();
                    match mutex {
                        Ok(mut rng) => {
                            let colors_chosen = choose(&prefs.0, &prefs.1, &game_id, &mut rng);
                            if let Err(_e) = self.reg.xadd.xadd(&colors_chosen) {
                                error!("error writing to colors chose stream")
                            }

                            info!("🎨 Completed: {:?}", colors_chosen)
                        }
                        Err(_) => todo!(),
                    }
                }
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }

            todo!("greetings")
        })
    }

    pub fn consume_choose_color_pref(&self, msg: &Message) -> anyhow::Result<()> {
        todo!()
    }
}

use redis_streams::{ConsumerGroupOpts, Group};
const BLOCK_MS: usize = 5000;
pub fn opts() -> ConsumerGroupOpts {
    ConsumerGroupOpts {
        block_ms: BLOCK_MS,
        group: Group {
            group_name: GROUP_NAME.to_string(),
            consumer_name: "singleton".to_string(),
        },
    }
}

pub fn process(components: &mut Components) {
    loop {
        // match components.xread.sorted() {
        //   Ok(xrr) => {
        //     for time_ordered_event in xrr {
        /*
        let (result, pxid) = match time_ordered_event {
            (xid, StreamInput::CCP(ccp)) => {
                info!("Stream: Choose Color Pref {:?}", &ccp);
                let scp = SessionColorPref {
                    color_pref: ccp.color_pref,
                    session_id: ccp.session_id.clone(),
                    client_id: ccp.client_id,
                };

                if let Err(_e) = components.prefs_repo.put(&scp) {
                    error!("write to pref repo")
                }

                (
                    game_color_prefs::by_session_id(&ccp.session_id, &repos),
                    Processed::CCP(xid),
                )
            }
            (xid, StreamInput::GR(gr)) => {
                info!("Stream: Game Ready {:?}", &gr);


            }
        };

        match result {
            Ok(GameColorPref::Complete { game_id, prefs }) => {
                let colors_chosen =
                    choose(&prefs.0, &prefs.1, &game_id, &mut components.random);
                if let Err(_e) = components.xadd.xadd(&colors_chosen) {
                    error!("error writing to colors chose stream")
                }

                info!("🎨 Completed: {:?}", colors_chosen)
            }
            Ok(other) => info!("⌚ Do Nothing: {:?}", other),
            Err(e) => error!("fetch error checking game color prefs {:?}", e),
        }

        match pxid {
            Processed::CCP(xid) => ccp_processed.push(xid),
            Processed::GR(xid) => gr_processed.push(xid),
        }
        */
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::repo::*;
    use crate::Components;
    use core_model::*;
    use crossbeam_channel::{unbounded, Receiver, Sender};
    use redis_streams::XId;
    use redis_streams::XReadGroupSorted;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    fn gr_to_msg(_: GameReady) -> Message {
        todo!()
    }

    fn ccp_to_msg(_: ChooseColorPref) -> Message {
        todo!()
    }

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
        max_read_millis: AtomicU64,
        sorted_data: Arc<Mutex<Vec<(XId, Message)>>>,
    }

    impl XReadGroupSorted for FakeXRead {
        fn read(
            &mut self,
            _stream_names: &[String],
        ) -> anyhow::Result<Vec<(XId, redis_streams::StreamMessage)>> {
            let max_ms = self.max_read_millis.load(Relaxed);
            /*let data: Vec<_> = self
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
            Ok(data)*/
            todo!()
        }
    }

    struct FakeXAdd(Sender<ColorsChosen>);
    impl XAdd for FakeXAdd {
        fn xadd(&self, data: &ColorsChosen) -> Result<(), XAddErr> {
            Ok(self.0.send(data.clone()).expect("send"))
        }
    }

    fn quick_xid(millis_time: u64) -> XId {
        XId {
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

    fn run_stream(events: Vec<Message>) -> TestOutputs {
        let gr_ack_ms = Arc::new(AtomicU64::new(0));
        let ccp_ack_ms = Arc::new(AtomicU64::new(0));

        let (xadd_call_in, xadd_call_out): (_, Receiver<ColorsChosen>) = unbounded();
        let (put_prefs_in, put_prefs_out): (_, Receiver<SessionColorPref>) = unbounded();
        let (put_game_ready_in, put_game_ready_out): (_, Receiver<GameReady>) = unbounded();

        let fake_prefs_contents: Arc<Mutex<HashMap<SessionId, SessionColorPref>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let fake_game_ready_contents: Arc<Mutex<HashMap<SessionId, GameReady>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let sorted_fake_stream: Arc<Mutex<Vec<(XId, Message)>>> = Arc::new(Mutex::new(vec![]));

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

            todo!("halp");
            /*match event {
                StreamInput::CCP(_) => assert_eq!(ccp_ack_ms.load(Relaxed), xid.millis_time),
                StreamInput::GR(_) => assert_eq!(gr_ack_ms.load(Relaxed), xid.millis_time),
            }*/
            todo!("what happened")
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

        let first_client_pref = ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0.clone(),
            color_pref: ColorPref::White,
        };
        let second_client_pref = ChooseColorPref {
            client_id: clients.1,
            session_id: sessions.1.clone(),
            color_pref: ColorPref::Black,
        };

        let board_size = 9;
        let game_ready = GameReady {
            game_id,
            sessions,
            event_id: EventId::new(),
            board_size,
        };
        let test_outputs = run_stream(vec![
            ccp_to_msg(first_client_pref),
            ccp_to_msg(second_client_pref),
            gr_to_msg(game_ready),
        ]);

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

        let first_client_pref = ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0.clone(),
            color_pref: ColorPref::White,
        };

        let second_client_pref = ChooseColorPref {
            client_id: clients.1,
            session_id: sessions.1.clone(),
            color_pref: ColorPref::Black,
        };

        let board_size = 9;

        let game_ready = GameReady {
            game_id,
            sessions,
            event_id: EventId::new(),
            board_size,
        };

        let test_outputs = run_stream(vec![
            gr_to_msg(game_ready),
            ccp_to_msg(first_client_pref),
            ccp_to_msg(second_client_pref),
        ]);

        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_game_ready_out.recv().expect("recv");
        test_outputs.xadd_call_out.recv().expect("recv");
    }

    #[test]
    fn no_game_ready_no_choice() {
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));

        let first_client_pref = ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0.clone(),
            color_pref: ColorPref::White,
        };
        let second_client_pref = ChooseColorPref {
            client_id: clients.1,
            session_id: sessions.1.clone(),
            color_pref: ColorPref::Black,
        };

        let test_outputs = run_stream(vec![
            ccp_to_msg(first_client_pref),
            ccp_to_msg(second_client_pref),
        ]);

        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_prefs_out.recv().expect("recv");
        assert!(test_outputs.xadd_call_out.is_empty());
    }

    #[test]
    fn no_full_pref_no_choice() {
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));

        let first_client_pref = ChooseColorPref {
            client_id: clients.0,
            session_id: sessions.0.clone(),
            color_pref: ColorPref::White,
        };
        let game_id = GameId::new();
        let board_size = 9;
        let game_ready = GameReady {
            game_id,
            sessions,
            event_id: EventId::new(),
            board_size,
        };

        let test_outputs = run_stream(vec![ccp_to_msg(first_client_pref), gr_to_msg(game_ready)]);

        test_outputs.put_prefs_out.recv().expect("recv");
        test_outputs.put_game_ready_out.recv().expect("recv");
        assert!(test_outputs.xadd_call_out.is_empty());
    }
}
