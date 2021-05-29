mod topics;
mod xadd;

pub use xadd::*;

use crate::components::*;
use crate::player::other_player;
use crate::sync::is_client_ahead_by_one_turn;
use crate::ToHistory;
use core_model::*;
use log::{error, info, trace, warn};
use move_model::*;
use redis_streams::SortedStreams;
use sync_model::api::*;
use sync_model::Move;
use sync_model::*;

const GROUP_NAME: &str = "micro-sync";

pub struct SyncStreams {
    pub components: Components,
}
impl SyncStreams {
    pub fn process(&self, streams: &mut dyn SortedStreams) {
        loop {
            if let Err(e) = streams.consume() {
                error!("Stream err {:?}", e)
            }
        }
    }

    fn process_prov_hist(&self, ph: &ProvideHistory) {
        info!("Stream: Provide History {:?}", ph);

        let components = &self.components;
        let maybe_hist_r = components.history_repo.get(&ph.game_id);
        match maybe_hist_r {
            Ok(Some(moves)) => {
                let hp = HistoryProvided {
                    moves,
                    event_id: EventId::new(),
                    epoch_millis: crate::time::now_millis() as u64,
                    game_id: ph.game_id.clone(),
                    reply_to: ph.req_id.clone(),
                };
                if let Err(e) = components.xadd.add_history_provided(hp) {
                    error!("error in xadd {:?}", e)
                }
            }
            Ok(None) => warn!("no history for game {:?}", ph.game_id),
            Err(e) => error!("history lookup error in prov hist: {:?}", e),
        }
    }

    fn process_game_state(&self, game_state: &GameState) {
        info!("Stream: Game State   {:?}", game_state.game_id);
        trace!("Full game state: {:?}", game_state);
        let components = &self.components;
        if let Err(_e) = components
            .history_repo
            .put(&game_state.game_id, game_state.to_history())
        {
            error!("write to history repo")
        }
    }

    fn process_move_made(&self, move_made: &MoveMade) {
        info!("Stream: Move Made {:?}", move_made);
        let components = &self.components;
        // Check ReplyOnMove repo to see if we have a req_sync associated with this
        // game_id & req_id combination.
        match components
            .reply_repo
            .get(&move_made.game_id, &move_made.reply_to)
        {
            Ok(Some(req_sync)) => {
                // We were waiting to hear about this move being made.
                //  we need to create a
                // sync reply based on this move.  This branch executes in the
                // case where a client was previously ahead of the backend and we
                // emitted a MakeMove request.  This MoveMade is the result
                // of changelog recording our move
                match components.history_repo.get(&move_made.game_id) {
                    Ok(history_nested) => {
                        let history = history_nested.unwrap_or_default();
                        let mut all_moves = history.clone();
                        let the_turn = (history.last().map(|l| l.turn)).unwrap_or_default() + 1;
                        all_moves.push(Move {
                            player: move_made.player,
                            coord: move_made.coord,
                            turn: the_turn,
                        });

                        let sync_reply = SyncReply {
                            session_id: req_sync.session_id,
                            game_id: req_sync.game_id,
                            reply_to: req_sync.req_id,
                            moves: all_moves,
                            turn: the_turn + 1,
                            player_up: other_player(move_made.player),
                        };
                        if let Err(_) = components.xadd.add_sync_reply(sync_reply) {
                            error!("xadd sync reply")
                        }
                    }
                    Err(e) => error!("history fetch err in move made processor: {:?}", e),
                }
            }
            Ok(None) => (),
            Err(e) => error!("error fetching from reply repo: {:?}", e),
        }
    }

    fn process_req_sync(&self, rs: &ReqSync) {
        info!("Stream: Req Sync {:?}", rs);
        let components = &self.components;
        match components.history_repo.get(&rs.game_id) {
            Ok(maybe_history) => {
                let history = maybe_history.unwrap_or_default();
                let system_last_move = history.last();
                let system_player_up = system_last_move
                    .map(|m| other_player(m.player))
                    .unwrap_or(Player::BLACK);
                let system_turn = system_last_move.map(|m| m.turn).unwrap_or(0) + 1;

                if is_client_ahead_by_one_turn(rs, system_turn, system_player_up) {
                    // client is ahead of server by a single turn
                    // and their move needs to be processed
                    if let Some(missed_move) = &rs.last_move {
                        let make_move = MakeMove {
                            game_id: rs.game_id.clone(),
                            req_id: rs.req_id.clone(),
                            player: missed_move.player,
                            coord: missed_move.coord,
                        };

                        if let Err(e) = components.xadd.add_make_move(make_move) {
                            error!("xadd make move {:?}", e)
                        }

                        // Very important ... 😈
                        // We need to remember this request, so that
                        // when a move is finally made by changelog,
                        // we don't forget to send a sync reply.

                        // The reply will ultimately be processed in our
                        // process loop's Message::MM(move_made) branch !
                        if let Err(_) = components.reply_repo.put(rs) {
                            error!("fail to put req in reply repo")
                        }
                    }
                } else {
                    // in every other case, we should send the server's view:
                    // - no op: client is caught up
                    // - client is behind by one move
                    // - client has a state which we cannot reconcile
                    //            ...(but maybe they can fix themselves)
                    let sync_reply = SyncReply {
                        moves: history,
                        game_id: rs.game_id.clone(),
                        reply_to: rs.req_id.clone(),
                        player_up: system_player_up,
                        turn: system_turn,
                        session_id: rs.session_id.clone(),
                    };
                    if let Err(e) = components.xadd.add_sync_reply(sync_reply) {
                        error!("xadd sync reply {:?}", e)
                    }
                }
            }
            Err(e) => error!("history lookup for req sync : {:?}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repo::*;
    use crate::Components;
    use crossbeam_channel::{select, unbounded, Receiver, Sender};
    use redis_streams::Message;
    use redis_streams::XId;
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

    struct FakeReplyRepo {
        pub contents: Arc<Mutex<Option<ReqSync>>>,
    }
    impl ReplyOnMoveRepo for FakeReplyRepo {
        fn get(&self, _game_id: &GameId, _req_id: &ReqId) -> Result<Option<ReqSync>, FetchErr> {
            Ok(self.contents.lock().expect("mutex").clone())
        }

        fn put(&self, req: &ReqSync) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            *data = Some(req.clone());
            Ok(())
        }

        fn del(&self, _game_id: &GameId, _req_id: &ReqId) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            *data = None;
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
        sorted_data: Arc<Mutex<Vec<(XId, Message)>>>,
        fake_acks: Arc<FakeAcks>,
    }

    struct StreamReadErr;
    trait XRead {
        fn read_sorted(&self) -> Result<Vec<(redis_streams::XId, Message)>, StreamReadErr>;
    }
    impl XRead for FakeXRead {
        fn read_sorted(&self) -> Result<Vec<(redis_streams::XId, Message)>, StreamReadErr> {
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
    }
    impl FakeXRead {
        fn update_max_id(&self, some: &AtomicU64, ids: &[XId]) {
            if let Some(max_id_millis) = ids.iter().map(|id| id.millis_time).max() {
                some.swap(max_id_millis, Ordering::Relaxed);
            }
        }
    }

    struct FakeXAdd {
        hist_prov_in: Sender<HistoryProvided>,
        sync_reply_in: Sender<SyncReply>,
        make_move_in: Sender<MakeMove>,
    }
    impl XAdd for FakeXAdd {
        fn add_history_provided(&self, data: HistoryProvided) -> Result<(), XAddErr> {
            Ok(self.hist_prov_in.send(data).expect("send"))
        }

        fn add_sync_reply(&self, data: SyncReply) -> Result<(), XAddErr> {
            Ok(self.sync_reply_in.send(data).expect("send"))
        }

        fn add_make_move(&self, data: MakeMove) -> Result<(), XAddErr> {
            Ok(self.make_move_in.send(data).expect("send"))
        }
    }

    fn quick_xid(ms: u64) -> XId {
        XId {
            millis_time: ms,
            seq_no: 0,
        }
    }

    const SLEEP_WAIT_MS: u64 = 100;
    struct TestFakes {
        history_contents: Arc<Mutex<Option<Vec<Move>>>>,
        reply_contents: Arc<Mutex<Option<ReqSync>>>,
        sorted_stream: Arc<Mutex<Vec<(XId, Message)>>>,
        sync_reply_xadd_out: Receiver<SyncReply>,
        hist_prov_xadd_out: Receiver<HistoryProvided>,
        make_move_xadd_out: Receiver<MakeMove>,
        acks: Arc<FakeAcks>,
        time_ms: u64,
    }
    impl TestFakes {
        pub fn emit_sleep(&mut self, input: Message) -> XId {
            let xid = quick_xid(self.time_ms);
            self.sorted_stream.lock().expect("lock").push((xid, input));

            let wait = self.wait();
            thread::sleep(wait);
            self.time_ms = self.time_ms + wait.as_millis() as u64;
            xid
        }

        pub fn wait(&self) -> Duration {
            Duration::from_millis(SLEEP_WAIT_MS)
        }
    }

    fn spawn_process_thread() -> TestFakes {
        let (hist_prov_xadd_in, hist_prov_xadd_out): (Sender<HistoryProvided>, _) = unbounded();
        let (sync_reply_xadd_in, sync_reply_xadd_out): (Sender<SyncReply>, _) = unbounded();
        let (make_move_xadd_in, make_move_xadd_out): (Sender<MakeMove>, _) = unbounded();

        let history_contents: Arc<Mutex<Option<Vec<Move>>>> = Arc::new(Mutex::new(None));
        let reply_contents: Arc<Mutex<Option<ReqSync>>> = Arc::new(Mutex::new(None));

        let sorted_stream: Arc<Mutex<Vec<(XId, Message)>>> = Arc::new(Mutex::new(vec![]));

        let sfs = sorted_stream.clone();
        let fh = history_contents.clone();
        let fr = reply_contents.clone();

        let acks = Arc::new(FakeAcks::new());
        let ackss = acks.clone();
        thread::spawn(move || {
            let components = Components {
                history_repo: Box::new(FakeHistoryRepo { contents: fh }),
                reply_repo: Box::new(FakeReplyRepo { contents: fr }),

                xadd: Box::new(FakeXAdd {
                    hist_prov_in: hist_prov_xadd_in,
                    sync_reply_in: sync_reply_xadd_in,
                    make_move_in: make_move_xadd_in,
                }),
            };
            todo!(); //process(&components);
        });

        TestFakes {
            history_contents,
            reply_contents,
            sorted_stream,
            hist_prov_xadd_out,
            sync_reply_xadd_out,
            make_move_xadd_out,
            acks,
            time_ms: 100, // be careful, this needs to be non-zero or everything will hang 😹
        }
    }

    /// Simple case.  Client is in sync with the server and
    /// will simply receive the server view.
    #[test]
    fn test_req_sync_no_op() {
        let mut fakes: TestFakes = spawn_process_thread();

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

        let game_id = GameId::new();
        let session_id = SessionId::new();
        let req_id = ReqId::new();

        // client is caught up to the backend
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

        let xid_rs = fakes.emit_sleep(todo!()); //Message::RS(req_sync));

        // request sync event should be acknowledged
        // during stream::process
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

        let actual = fakes.sync_reply_xadd_out.recv().expect("recv");
        assert_eq!(actual, expected)
    }

    /// Simple case.  Client is behind by one move and needs to catch up
    /// to the server.
    #[test]
    fn test_req_sync_client_catch_up() {
        let mut fakes = spawn_process_thread();

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

        let game_id = GameId::new();
        let session_id = SessionId::new();
        let req_id = ReqId::new();

        // client view is behind by one move
        let client_last_move_behind_by_one = moves[0].clone();

        let req_sync = ReqSync {
            session_id: session_id.clone(),
            req_id: req_id.clone(),
            game_id: game_id.clone(),
            last_move: Some(client_last_move_behind_by_one),
            player_up: Player::WHITE, // behind by one
            turn: turn - 1,           // behind by one
        };

        // set contents of fake history repo.
        // note that it is one move ahead of the client's view
        *fakes.history_contents.lock().expect("lock") = Some(moves.clone());

        let xid_rs = fakes.emit_sleep(todo!()); //Message::RS(req_sync));

        // request sync event should be acknowledged
        // during stream::process
        let rs_ack = fakes.acks.last_rs_ack_ms.load(Ordering::Relaxed);
        assert_eq!(rs_ack, xid_rs.millis_time);

        // system replies so that client can catch
        // up to current status
        let expected = SyncReply {
            session_id,
            reply_to: req_id,
            moves,
            game_id,
            player_up,
            turn,
        };

        let actual = fakes.sync_reply_xadd_out.recv().expect("recv");
        assert_eq!(actual, expected)
    }

    /// Client sends a turn / lastMove combination that
    /// seems completely bogus.  Server responds with the
    /// correct view of the game (same as no-op, or client
    /// being behind).  If  the client is able to fix their
    /// local state, that's great.  If not, sync has still
    /// done its job.
    #[test]
    fn test_req_sync_bogus_client_state() {
        let mut fakes = spawn_process_thread();

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

        let game_id = GameId::new();
        let session_id = SessionId::new();
        let req_id = ReqId::new();

        let bogus_client_turn = 7;
        let bogus_client_move = Move {
            player: Player::BLACK,
            coord: Some(Coord { x: 13, y: 13 }),
            turn: bogus_client_turn,
        };

        let req_sync: ReqSync = ReqSync {
            session_id: session_id.clone(),
            req_id: req_id.clone(),
            game_id: game_id.clone(),
            player_up: Player::BLACK,
            turn: bogus_client_turn,
            last_move: Some(bogus_client_move),
        };

        // make sure fake history repo is configured
        *fakes.history_contents.lock().expect("lock") = Some(moves.clone());

        let xid_rs = fakes.emit_sleep(todo!()); //Message::RS(req_sync));

        // request sync event should be acknowledged
        // during stream::process
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
        let actual = fakes.sync_reply_xadd_out.recv().expect("recv");
        assert_eq!(actual, expected)
    }
    #[test]
    fn test_req_sync_server_catch_up() {
        let client_turn = 4;
        let client_player_up = Player::WHITE;

        let client_moves = vec![
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
            Move {
                player: Player::BLACK,
                coord: Some(Coord { x: 4, y: 5 }),
                turn: 3,
            },
        ];
        let server_moves = vec![
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

        let game_id = GameId::new();
        let session_id = SessionId::new();
        let req_id = ReqId::new();

        let client_last_move = client_moves.last().map(|m| m.clone());
        let req_sync: ReqSync = ReqSync {
            game_id: game_id.clone(),
            session_id: session_id.clone(),
            req_id: req_id.clone(),
            last_move: client_last_move.clone(),
            turn: client_turn,
            player_up: client_player_up,
        };

        let mut fakes: TestFakes = spawn_process_thread();

        // make sure fake history repo is configured
        *fakes.history_contents.lock().expect("lock") = Some(server_moves.clone());

        let xid_rs = fakes.emit_sleep(todo!()); //Message::RS(req_sync.clone()));

        // request sync event should be acknowledged
        // during stream::process
        let rs_ack = fakes.acks.last_rs_ack_ms.load(Ordering::Relaxed);
        assert_eq!(rs_ack, xid_rs.millis_time);

        let actual_req_saved = fakes
            .reply_contents
            .lock()
            .expect("lock")
            .as_ref()
            .expect("some")
            .clone();
        assert_eq!(actual_req_saved, req_sync);

        // There should be an xadd to make-move-ev
        let actual_make_move = fakes.make_move_xadd_out.recv().expect("recv xadd");
        let expected_make_move = MakeMove {
            game_id: game_id.clone(),
            req_id: req_id.clone(),
            player: other_player(client_player_up),
            coord: client_last_move.clone().expect("client last move").coord,
        };
        assert_eq!(actual_make_move.game_id, expected_make_move.game_id);
        assert_eq!(actual_make_move.req_id, expected_make_move.req_id);
        assert_eq!(actual_make_move.player, expected_make_move.player);
        assert_eq!(actual_make_move.coord, expected_make_move.coord);

        let move_made_at_changelog = MoveMade {
            game_id: game_id.clone(),
            reply_to: req_id.clone(),
            coord: client_last_move.clone().expect("move").coord,
            player: client_last_move.expect("move").player,
            event_id: EventId::new(),
            captured: Vec::new(),
        };
        let xid_mm = fakes.emit_sleep(todo!()); //Message::MM(move_made_at_changelog));

        let mm_ack = fakes.acks.last_mm_ack_ms.load(Ordering::Relaxed);
        assert_eq!(mm_ack, xid_mm.millis_time);

        let expected: SyncReply = SyncReply {
            session_id,
            reply_to: req_id,
            moves: client_moves,
            game_id,
            player_up: client_player_up,
            turn: client_turn,
        };
        let actual = fakes.sync_reply_xadd_out.recv().expect("recv");
        assert_eq!(actual, expected)
    }

    /// Test the ProvideHistory API
    #[test]
    fn test_provide_history() {
        let mut fakes = spawn_process_thread();

        // emit some events in a time-ordered fashion
        // (fake xread impl expects time ordering 😁)
        let fake_game_id = GameId(uuid::Uuid::default());
        let fake_moves = vec![
            MoveMade {
                player: Player::BLACK,
                coord: Some(Coord { x: 1, y: 1 }),
                game_id: fake_game_id.clone(),
                reply_to: ReqId::new(),
                event_id: EventId::new(),
                captured: Vec::new(),
            },
            MoveMade {
                player: Player::WHITE,
                coord: None,
                reply_to: ReqId::new(),
                event_id: EventId::new(),
                game_id: fake_game_id.clone(),
                captured: Vec::new(),
            },
        ];
        let fake_player_up = Player::BLACK;
        let xid_gs = fakes.emit_sleep(todo!()); /*Message::GS(GameState {
                                                    moves: fake_moves,
                                                    player_up: fake_player_up,
                                                    board: Board::default(),
                                                    captures: Captures::default(),
                                                    game_id: fake_game_id.clone(),
                                                    turn: 1,
                                                }));*/

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
        let xid_ph = fakes.emit_sleep(todo!()); /*Message::PH(ProvideHistory {
                                                    game_id: fake_game_id.clone(),
                                                    req_id: fake_req_id.clone(),
                                                }));*/

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
            default(fakes.wait()) => panic!("WAIT timeout")
        }
    }
}
