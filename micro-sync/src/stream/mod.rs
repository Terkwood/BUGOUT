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
    RS(ReqSync),
    MM(MoveMade),
}

pub fn process(components: &Components) {
    let mut unacked = Unacknowledged::default();
    loop {
        match components.xread.read_sorted() {
            Ok(xrr) => {
                for time_ordered_event in xrr {
                    match time_ordered_event {
                        (xid, StreamInput::RS(rs)) => {
                            todo!(" stream match req sync");
                            unacked.req_sync.push(xid)
                        }
                        (xid, StreamInput::PH(ProvideHistory { game_id, req_id })) => {
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
                                    if let Err(e) = components.xadd.add_history_provided(hp) {
                                        error!("error in xadd {:?}", e)
                                    }
                                }
                                Ok(None) => warn!("no history for game {:?}", game_id),
                                Err(_e) => error!("history lookup error"),
                            }

                            unacked.prov_hist.push(xid);
                        }
                        (xid, StreamInput::GS(game_id, game_state)) => {
                            if let Err(_e) = components
                                .history_repo
                                .put(&game_id, game_state.to_history())
                            {
                                error!("write to history repo")
                            }

                            unacked.game_states.push(xid)
                        }
                        (xid, StreamInput::MM(_)) => {
                            todo!("stream match move made");
                            unacked.move_made.push(xid)
                        }
                    }
                }
            }
            Err(_) => error!("xread"),
        }

        unacked.ack_all(&components)
    }
}

struct Unacknowledged {
    req_sync: Vec<XReadEntryId>,
    prov_hist: Vec<XReadEntryId>,
    game_states: Vec<XReadEntryId>,
    move_made: Vec<XReadEntryId>,
}

const GROUP_NAME: &str = "micro-sync";

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

    static MAX_READ_XID_MILLIS: AtomicU64 = AtomicU64::new(0);
    static LAST_GS_ACK_MILLIS: AtomicU64 = AtomicU64::new(0);
    static LAST_RS_ACK_MILLIS: AtomicU64 = AtomicU64::new(0);
    static LAST_PH_ACK_MILLIS: AtomicU64 = AtomicU64::new(0);
    static LAST_MM_ACK_MILLIS: AtomicU64 = AtomicU64::new(0);

    struct FakeHistoryRepo {
        pub contents: Arc<Mutex<Option<Vec<Move>>>>,
    }

    impl HistoryRepo for FakeHistoryRepo {
        fn get(&self, _game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr> {
            Ok(self.contents.lock().expect("mutex").clone())
        }

        fn put(&self, _game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            *data = Some(moves.clone());
            Ok(())
        }
    }

    struct FakeXRead {
        sorted_data: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
    }
    impl XRead for FakeXRead {
        fn read_sorted(
            &self,
        ) -> Result<Vec<(redis_streams::XReadEntryId, StreamInput)>, StreamReadErr> {
            let max_xid_ms = MAX_READ_XID_MILLIS.load(Ordering::Relaxed);

            let data: Vec<_> = self
                .sorted_data
                .lock()
                .expect("lock")
                .iter()
                .filter(|(xid, _)| max_xid_ms < xid.millis_time)
                .cloned()
                .collect();

            if data.is_empty() {
                // stop the test thread from spinning like crazy
                std::thread::sleep(Duration::from_millis(20))
            } else {
                // this hack is standing in for "xreadgroup >" semantics
                let new_max_xid_ms = data.iter().map(|(eid, _)| eid).max().unwrap();
                MAX_READ_XID_MILLIS.swap(new_max_xid_ms.millis_time, Ordering::Relaxed);
            }
            Ok(data)
        }

        fn ack_req_sync(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            Ok(self.update_max_id(&LAST_RS_ACK_MILLIS, ids))
        }

        fn ack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            Ok(self.update_max_id(&LAST_PH_ACK_MILLIS, ids))
        }

        fn ack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            Ok(self.update_max_id(&LAST_GS_ACK_MILLIS, ids))
        }

        fn ack_move_made(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            Ok(self.update_max_id(&LAST_MM_ACK_MILLIS, ids))
        }
    }
    impl FakeXRead {
        fn update_max_id(&self, some: &AtomicU64, ids: &[XReadEntryId]) {
            if let Some(max_id_millis) = ids.iter().map(|id| id.millis_time).max() {
                some.swap(max_id_millis, Ordering::Relaxed);
            }
        }
    }

    struct FakeXAdd {
        hist_prov_in: Sender<HistoryProvided>,
        sync_reply_in: Sender<SyncReply>,
    }
    impl XAdd for FakeXAdd {
        fn add_history_provided(&self, data: HistoryProvided) -> Result<(), XAddErr> {
            Ok(self.hist_prov_in.send(data).expect("send"))
        }

        fn add_sync_reply(&self, data: SyncReply) -> Result<(), XAddErr> {
            Ok(self.sync_reply_in.send(data).expect("send"))
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
        let (hist_prov_xadd_in, hist_prov_xadd_out): (Sender<HistoryProvided>, _) = unbounded();
        let (sync_reply_xadd_in, sync_reply_xadd_out): (Sender<SyncReply>, _) = unbounded();

        // set up a loop to process game lobby requests
        let fake_history_contents = Arc::new(Mutex::new(None));

        let sorted_fake_stream: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>> =
            Arc::new(Mutex::new(vec![]));

        let sfs = sorted_fake_stream.clone();
        let fh = fake_history_contents.clone();
        thread::spawn(move || {
            let components = Components {
                history_repo: Box::new(FakeHistoryRepo { contents: fh }),
                xread: Box::new(FakeXRead {
                    sorted_data: sfs.clone(),
                }),
                xadd: Box::new(FakeXAdd {
                    hist_prov_in: hist_prov_xadd_in,
                    sync_reply_in: sync_reply_xadd_in,
                }),
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
            recv(hist_prov_xadd_out) -> msg => match msg {
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

impl Unacknowledged {
    pub fn ack_all(&mut self, components: &Components) {
        if !self.req_sync.is_empty() {
            if let Err(_e) = components.xread.ack_req_sync(&self.req_sync) {
                error!("ack for req sync failed")
            } else {
                self.req_sync.clear();
            }
        }

        if !self.prov_hist.is_empty() {
            if let Err(_e) = components.xread.ack_prov_hist(&self.prov_hist) {
                error!("ack for provide history failed")
            } else {
                self.prov_hist.clear();
            }
        }
        if !self.game_states.is_empty() {
            if let Err(_e) = components.xread.ack_game_states(&self.game_states) {
                error!("ack for game states failed")
            } else {
                self.game_states.clear();
            }
        }
        if !self.move_made.is_empty() {
            if let Err(_e) = components.xread.ack_move_made(&self.move_made) {
                error!("ack for move made failed")
            } else {
                self.move_made.clear();
            }
        }
    }
}

const INIT_ACK_CAPACITY: usize = 25;
impl Default for Unacknowledged {
    fn default() -> Self {
        Self {
            prov_hist: Vec::with_capacity(INIT_ACK_CAPACITY),
            req_sync: Vec::with_capacity(INIT_ACK_CAPACITY),
            game_states: Vec::with_capacity(INIT_ACK_CAPACITY),
            move_made: Vec::with_capacity(INIT_ACK_CAPACITY),
        }
    }
}
