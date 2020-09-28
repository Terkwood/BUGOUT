pub mod init;
mod topics;
mod xadd;
mod xread;

pub use xadd::*;
pub use xread::*;

use crate::api::*;
use crate::components::*;
use crate::model::*;
use crate::sync::is_client_ahead_by_one_turn;
use log::{error, warn};
use redis_streams::XReadEntryId;

const GROUP_NAME: &str = "micro-sync";

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
                for (xid, event) in xrr {
                    process_event(xid, &event, components);
                    unacked.push(xid, event);
                }
            }
            Err(_) => error!("xread"),
        }

        unacked.ack_all(&components)
    }
}

fn process_event(xid: XReadEntryId, event: &StreamInput, components: &Components) {
    match event {
        StreamInput::RS(rs) => {
            match components.history_repo.get(&rs.game_id) {
                Ok(maybe_history) => {
                    let history = maybe_history.unwrap_or_default();
                    let system_last_move = history.last();
                    let system_player_up = system_last_move
                        .map(|m| other_player(m.player))
                        .unwrap_or(Player::BLACK);
                    let system_turn = system_last_move.map(|m| m.turn).unwrap_or(0) + 1;

                    if is_client_ahead_by_one_turn(rs, history, system_turn, system_player_up) {
                        todo!()
                    } else {
                        todo!()
                    }
                }
                Err(_) => error!("history lookup for req sync"),
            }
            /*

                    // THIS PORTION WILL BE INSPECTED DOWN STREAM
                    HistProvReply(left,
                            right,
                            systemTurn, systemPlayerUp
                    )

                    val branches = histReplyStream.kbranch(
                // client is ahead of server by a single turn
                // and their move needs to be processed
                { _: SessionId, hpr: HistProvReply ->
                    isClientAheadByOneTurn(hpr)
                },
                // in every other case, we should send the server's view:
                // - no op: client is caught up
                // - client is behind by one move
                // - client has a state which we cannot reconcile
                //            ...(but maybe they can fix themselves)
                { _: SessionId, hpr: HistProvReply ->
                    !isClientAheadByOneTurn(hpr) }
            )

            val clientAheadByOneTurnBranch = branches[0]
            clientAheadByOneTurnBranch.map { _, v ->
                    val missedMove = v.reqSync.lastMove!! // checked null above
                    KeyValue(v.reqSync.gameId,
                        MakeMoveCmd(gameId = v
                            .reqSync.gameId, reqId = v.reqSync.reqId,
                            player = missedMove.player,
                            coord = missedMove.coord)) }
                .mapValues { v ->
                    jsonMapper.writeValueAsString(v)
                }
                .to(Topics.MAKE_MOVE_CMD,
                    Produced.with(Serdes.UUID(), Serdes.String()))

                },
                            */
        }
        StreamInput::PH(ProvideHistory { game_id, req_id }) => {
            let maybe_hist_r = components.history_repo.get(&game_id);
            match maybe_hist_r {
                Ok(Some(moves)) => {
                    let hp = HistoryProvided {
                        moves,
                        event_id: EventId::new(),
                        epoch_millis: crate::time::now_millis() as u64,
                        game_id: game_id.clone(),
                        reply_to: req_id.clone(),
                    };
                    if let Err(e) = components.xadd.add_history_provided(hp) {
                        error!("error in xadd {:?}", e)
                    }
                }
                Ok(None) => warn!("no history for game {:?}", game_id),
                Err(_e) => error!("history lookup error"),
            }
        }
        StreamInput::GS(game_id, game_state) => {
            if let Err(_e) = components
                .history_repo
                .put(&game_id, game_state.to_history())
            {
                error!("write to history repo")
            }
        }
        StreamInput::MM(_) => {
            todo!("stream match move made");
        }
    }
}

fn other_player(player: Player) -> Player {
    match player {
        Player::BLACK => Player::WHITE,
        Player::WHITE => Player::BLACK,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repo::*;
    use crate::Components;
    use crossbeam_channel::{select, unbounded, Receiver, Sender};
    use redis_streams::XReadEntryId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

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

    struct FakeAcks {
        last_mm_ack_ms: AtomicU64,
        last_rs_ack_ms: AtomicU64,
        last_ph_ack_ms: AtomicU64,
        last_gs_ack_ms: AtomicU64,
        max_read_xid_ms: AtomicU64,
    }
    impl FakeAcks {
        pub fn new() -> Self {
            Self {
                last_mm_ack_ms: AtomicU64::new(0),
                last_rs_ack_ms: AtomicU64::new(0),
                last_ph_ack_ms: AtomicU64::new(0),
                last_gs_ack_ms: AtomicU64::new(0),
                max_read_xid_ms: AtomicU64::new(0),
            }
        }
    }
    struct FakeXRead {
        sorted_data: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
        fake_acks: Arc<FakeAcks>,
    }
    impl XRead for FakeXRead {
        fn read_sorted(
            &self,
        ) -> Result<Vec<(redis_streams::XReadEntryId, StreamInput)>, StreamReadErr> {
            let max_xid_ms = self.fake_acks.max_read_xid_ms.load(Ordering::Relaxed);

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
                self.fake_acks
                    .max_read_xid_ms
                    .swap(new_max_xid_ms.millis_time, Ordering::Relaxed);
            }
            Ok(data)
        }

        fn ack_req_sync(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            Ok(self.update_max_id(&self.fake_acks.last_rs_ack_ms, ids))
        }

        fn ack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            Ok(self.update_max_id(&self.fake_acks.last_ph_ack_ms, ids))
        }

        fn ack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            Ok(self.update_max_id(&self.fake_acks.last_gs_ack_ms, ids))
        }

        fn ack_move_made(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
            Ok(self.update_max_id(&self.fake_acks.last_mm_ack_ms, ids))
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

        fn add_make_move(&self, data: MakeMove) -> Result<(), XAddErr> {
            todo!()
        }
    }

    fn quick_eid(ms: u64) -> XReadEntryId {
        XReadEntryId {
            millis_time: ms,
            seq_no: 0,
        }
    }

    struct TestFakes {
        history_contents: Arc<Mutex<Option<Vec<Move>>>>,
        sorted_stream: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
        sync_reply_xadd_out: Receiver<SyncReply>,
        hist_prov_xadd_out: Receiver<HistoryProvided>,
        acks: Arc<FakeAcks>,
    }

    fn spawn_process_thread() -> TestFakes {
        let (hist_prov_xadd_in, hist_prov_xadd_out): (Sender<HistoryProvided>, _) = unbounded();
        let (sync_reply_xadd_in, sync_reply_xadd_out): (Sender<SyncReply>, _) = unbounded();

        let history_contents: Arc<Mutex<Option<Vec<Move>>>> = Arc::new(Mutex::new(None));

        let sorted_stream: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>> =
            Arc::new(Mutex::new(vec![]));

        let sfs = sorted_stream.clone();
        let fh = history_contents.clone();

        let acks = Arc::new(FakeAcks::new());
        let ackss = acks.clone();
        thread::spawn(move || {
            let components = Components {
                history_repo: Box::new(FakeHistoryRepo { contents: fh }),
                xread: Box::new(FakeXRead {
                    sorted_data: sfs.clone(),
                    fake_acks: ackss,
                }),
                xadd: Box::new(FakeXAdd {
                    hist_prov_in: hist_prov_xadd_in,
                    sync_reply_in: sync_reply_xadd_in,
                }),
            };
            process(&components);
        });

        TestFakes {
            history_contents,
            sorted_stream,
            hist_prov_xadd_out,
            sync_reply_xadd_out,
            acks,
        }
    }

    /// Simple case.  Client is in sync with the server and
    /// will simply receive the server view.
    #[test]
    fn test_req_sync_no_op() {
        let fakes: TestFakes = spawn_process_thread();

        let turn = 3;
        let player_up = Player::BLACK;
        let moves = vec![
            Move {
                player: Player::BLACK,
                coord: Some(Coord { x: 4, y: 4 }),
                turn: 1,
            },
            Move {
                player: Player::WHITE,
                coord: Some(Coord { x: 10, y: 10 }),
                turn: 2,
            },
        ];

        let game_id = GameId::random();
        let session_id = SessionId::random();
        let req_id = ReqId::random();

        let last_move = moves.last().cloned();
        let req_sync = ReqSync {
            session_id: session_id.clone(),
            req_id: req_id.clone(),
            game_id: game_id.clone(),
            last_move,
            player_up,
            turn,
        };

        // force fake history repo to respond as we expect
        *fakes.history_contents.lock().expect("lock") = Some(moves.clone());

        let wait = Duration::from_millis(166);
        let mut fake_time_ms = 100;
        let incr_ms = 100;

        let xid_rs = quick_eid(fake_time_ms);
        // emit a request for sync
        fakes
            .sorted_stream
            .lock()
            .expect("lock")
            .push((xid_rs, StreamInput::RS(req_sync)));
        fake_time_ms += incr_ms;

        thread::sleep(wait);

        let rs_ack = fakes.acks.last_rs_ack_ms.load(Ordering::Relaxed);
        assert_eq!(rs_ack, xid_rs.millis_time);

        let expected = SyncReply {
            session_id,
            reply_to: req_id,
            moves,
            game_id,
            player_up,
            turn,
        };

        todo!("draft test")
    }

    #[test]
    fn test_req_sync_client_catch_up() {
        let fakes = spawn_process_thread();
        todo!("draft test")
    }
    #[test]
    fn test_req_sync_bogus_client_state() {
        let fakes = spawn_process_thread();
        todo!("draft test")
    }
    #[test]
    fn test_req_sync_server_catch_up() {
        let fakes = spawn_process_thread();
        todo!("draft test")
    }

    /// Test the ProvideHistory API
    #[test]
    fn test_provide_history() {
        let fakes = spawn_process_thread();

        // emit some events in a time-ordered fashion
        // (fake xread impl expects time ordering 😁)

        let wait = Duration::from_millis(166);
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
        let xid_gs = quick_eid(fake_time_ms);
        // emit a game state
        fakes.sorted_stream.lock().expect("lock").push((
            xid_gs,
            StreamInput::GS(
                fake_game_id.clone(),
                GameState {
                    moves: Some(fake_moves),
                    player_up: fake_player_up,
                },
            ),
        ));
        fake_time_ms += incr_ms;

        thread::sleep(wait);

        // history repo should now contain the moves from that game
        let actual_moves = fakes
            .history_contents
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
        let gs_ack = fakes.acks.last_gs_ack_ms.load(Ordering::Relaxed);
        assert_eq!(gs_ack, xid_gs.millis_time);

        // request history
        let fake_req_id = ReqId(uuid::Uuid::default());
        let xid_ph = quick_eid(fake_time_ms);
        fakes.sorted_stream.lock().expect("lock").push((
            xid_ph,
            StreamInput::PH(ProvideHistory {
                game_id: fake_game_id.clone(),
                req_id: fake_req_id.clone(),
            }),
        ));

        // There should be an XADD triggered on history-provided stream
        select! {
            recv(fakes.hist_prov_xadd_out) -> msg => match msg {
                Ok(HistoryProvided { game_id, reply_to, moves, event_id: _, epoch_millis: _, }) => {
                    assert_eq!(game_id, fake_game_id);
                    assert_eq!(moves, expected_moves);
                    assert_eq!(reply_to, fake_req_id);
                    // check ack for provide_history stream
                    let ph_ack = fakes.acks.last_ph_ack_ms.load(Ordering::Relaxed);
                    assert_eq!(ph_ack, xid_ph.millis_time)
                },
                _ => panic!("wrong output")
            },
            default(wait) => panic!("WAIT timeout")
        }
    }
}
