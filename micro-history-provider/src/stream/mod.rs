mod topics;
mod xadd;
mod xread;

pub use xadd::*;
pub use xread::*;

use crate::api::*;
use crate::components::*;
use crate::model::*;
use log::{error, warn};
use redis::Commands;
use redis_streams::XReadEntryId;

#[derive(Clone, Debug)]
pub enum StreamInput {
    PH(ProvideHistory),
    GS(GameId, GameState),
}

pub fn process(components: &Components) {
    let mut gs_processed: Vec<XReadEntryId> = vec![];
    let mut ph_processed: Vec<XReadEntryId> = vec![];
    loop {
        match components.xread.xread_sorted() {
            Ok(xrr) => {
                for time_ordered_event in xrr {
                    match time_ordered_event {
                        (entry_id, StreamInput::GS(game_id, game_state)) => {
                            if let Err(_e) = components
                                .history_repo
                                .put(&game_id, game_state.to_history())
                            {
                                error!("write to history repo")
                            }

                            gs_processed.push(entry_id)
                        }
                        (entry_id, StreamInput::PH(ProvideHistory { game_id, req_id })) => {
                            let maybe_hist_r = components.history_repo.get(&game_id);
                            match maybe_hist_r {
                                Ok(Some(moves)) => {
                                    let hp = HistoryProvided {
                                        moves,
                                        event_id: EventId::new(),
                                        epoch_millis: crate::time::now_millis() as u64,
                                        game_id,
                                        reply_to: req_id,
                                    };
                                    if let Err(e) = components.xadd.xadd(hp) {
                                        error!("error in xadd {:?}", e)
                                    }
                                }
                                Ok(None) => warn!("no history for game {:?}", game_id),
                                Err(_e) => error!("history lookup error"),
                            }

                            ph_processed.push(entry_id);
                        }
                    }
                }
            }
            Err(_) => error!("xread"),
        }
        if !gs_processed.is_empty() {
            if let Err(_e) = components.xread.xack_game_states(&gs_processed) {
                error!("ack for game states failed")
            }
        }
        if !ph_processed.is_empty() {
            if let Err(_e) = components.xread.xack_prov_hist(&ph_processed) {
                error!("ack for provide history failed")
            }
        }
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
    use crossbeam_channel::{select, unbounded, Sender};
    use redis_streams::XReadEntryId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    static MAX_READ_EID_MILLIS: AtomicU64 = AtomicU64::new(0);
    static LAST_GS_ACK_MILLIS: AtomicU64 = AtomicU64::new(0);
    static LAST_PH_ACK_MILLIS: AtomicU64 = AtomicU64::new(0);

    struct FakeHistoryRepo {
        pub contents: Arc<Mutex<Option<Vec<Move>>>>,
        pub put_in: Sender<Vec<Move>>,
    }

    impl HistoryRepo for FakeHistoryRepo {
        fn get(&self, _game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr> {
            Ok(self.contents.lock().expect("mutex").clone())
        }

        fn put(&self, _game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            *data = Some(moves.clone());
            Ok(self.put_in.send(moves).expect("send"))
        }
    }

    struct FakeXRead {
        sorted_data: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
    }
    impl XRead for FakeXRead {
        fn xread_sorted(
            &self,
        ) -> Result<Vec<(redis_streams::XReadEntryId, StreamInput)>, StreamReadErr> {
            let max_eid_millis = MAX_READ_EID_MILLIS.load(Ordering::Relaxed);

            let data: Vec<_> = self
                .sorted_data
                .lock()
                .expect("lock")
                .iter()
                .filter(|(eid, stream_data)| match stream_data {
                    StreamInput::PH(_) => max_eid_millis < eid.millis_time,
                    StreamInput::GS(_, _) => max_eid_millis < eid.millis_time,
                })
                .cloned()
                .collect();

            if data.is_empty() {
                // stop the test thread from spinning like crazy
                std::thread::sleep(Duration::from_millis(20))
            } else {
                // this hack is standing in for "xreadgroup >" semantics
                let new_max_eid_millis = data.iter().map(|(eid, _)| eid).max().unwrap();
                MAX_READ_EID_MILLIS.swap(new_max_eid_millis.millis_time, Ordering::Relaxed);
            }
            Ok(data)
        }

        fn xack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            if let Some(max_id_millis) = ids.iter().map(|id| id.millis_time).max() {
                LAST_PH_ACK_MILLIS.swap(max_id_millis, Ordering::Relaxed);
            }

            Ok(())
        }

        fn xack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            if let Some(max_id_millis) = ids.iter().map(|id| id.millis_time).max() {
                LAST_GS_ACK_MILLIS.swap(max_id_millis, Ordering::Relaxed);
            }

            Ok(())
        }
    }

    struct FakeXAdd(Sender<HistoryProvided>);
    impl XAdd for FakeXAdd {
        fn xadd(&self, data: HistoryProvided) -> Result<(), XAddErr> {
            Ok(self.0.send(data).expect("send"))
        }
    }

    fn quick_eid(ms: u64) -> XReadEntryId {
        XReadEntryId {
            millis_time: ms,
            seq_no: 0,
        }
    }

    #[test]
    fn test_process() {
        let (xadd_call_in, xadd_call_out): (Sender<HistoryProvided>, _) = unbounded();
        let (put_history_in, put_history_out): (Sender<Vec<Move>>, _) = unbounded();

        // set up a loop to process game lobby requests
        let fake_history_contents = Arc::new(Mutex::new(None));
        let fh = fake_history_contents.clone();
        std::thread::spawn(move || loop {
            select! {
                recv(put_history_out) -> msg => match msg {
                    Ok(moves) => *fh.lock().expect("mutex lock") = Some(moves),
                    Err(_) => panic!("fail")
                }
            }
        });

        let sorted_fake_stream: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>> =
            Arc::new(Mutex::new(vec![]));

        let sfs = sorted_fake_stream.clone();
        let fh = fake_history_contents.clone();
        thread::spawn(move || {
            let components = Components {
                history_repo: Box::new(FakeHistoryRepo {
                    contents: fh,
                    put_in: put_history_in,
                }),
                xread: Box::new(FakeXRead {
                    sorted_data: sfs.clone(),
                }),
                xadd: Box::new(FakeXAdd(xadd_call_in)),
            };
            process(&components);
        });

        // emit some events in a time-ordered fashion
        // (fake xread impl expects time ordering 😁)

        let timeout = Duration::from_millis(166);
        let mut fake_time_ms = 100;
        let incr_ms = 100;

        let fake_game_id = GameId(uuid::Uuid::default());
        let fake_moves = vec![
            MoveEvent {
                player: Player::BLACK,
                coord: Some(Coord { x: 1, y: 1 }),
            },
            MoveEvent {
                player: Player::WHITE,
                coord: None,
            },
        ];
        let fake_player_up = Player::BLACK;
        let eid_gs = quick_eid(fake_time_ms);
        // emit a game state
        sorted_fake_stream.lock().expect("lock").push((
            eid_gs,
            StreamInput::GS(
                fake_game_id.clone(),
                GameState {
                    moves: Some(fake_moves),
                    player_up: fake_player_up,
                },
            ),
        ));
        fake_time_ms += incr_ms;

        thread::sleep(timeout);

        // history repo should now contain the moves from that game
        let actual_moves = fake_history_contents
            .clone()
            .lock()
            .expect("hr")
            .clone()
            .unwrap();
        let expected_moves = vec![
            Move {
                player: Player::BLACK,
                coord: Some(Coord { x: 1, y: 1 }),
                turn: 1,
            },
            Move {
                player: Player::WHITE,
                coord: None,
                turn: 2,
            },
        ];
        assert_eq!(actual_moves, expected_moves);
        // check ack for game_states stream
        let gs_ack = LAST_GS_ACK_MILLIS.load(Ordering::Relaxed);
        assert_eq!(gs_ack, eid_gs.millis_time);

        // request history
        let fake_req_id = ReqId(uuid::Uuid::default());
        let eid_ph = quick_eid(fake_time_ms);
        sorted_fake_stream.lock().expect("lock").push((
            eid_ph,
            StreamInput::PH(ProvideHistory {
                game_id: fake_game_id.clone(),
                req_id: fake_req_id.clone(),
            }),
        ));

        // There should be an XADD triggered on history-provided stream
        select! {
            recv(xadd_call_out) -> msg => match msg {
                Ok(HistoryProvided { game_id, reply_to, moves, event_id: _, epoch_millis: _, }) => {
                    assert_eq!(game_id, fake_game_id);
                    assert_eq!(moves, expected_moves);
                    assert_eq!(reply_to, fake_req_id);
                    // check ack for provide_history stream
                    let ph_ack = LAST_PH_ACK_MILLIS.load(Ordering::Relaxed);
                    assert_eq!(ph_ack, eid_ph.millis_time)
                },
                _ => panic!("wrong output")
            },
            default(timeout) => panic!("WAIT timeout")
        }
    }
}
