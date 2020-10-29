pub mod init;
mod input;
mod opts;
pub mod topics;
mod unack;
mod write_moves;
pub mod xack;
pub mod xadd;
pub mod xread;

pub use opts::StreamOpts;
pub use unack::Unacknowledged;
pub use write_moves::xadd_loop;

use crate::max_visits;
use bot_model::api::{AttachBot, ComputeMove};
pub use input::StreamInput;
use log::{error, info};
use move_model::GameState;

const GROUP_NAME: &str = "botlink";

pub fn xread_loop(opts: &mut StreamOpts) {
    let mut unack = Unacknowledged::default();
    loop {
        match opts.xread.xread_sorted() {
            Ok(xrr) => {
                for (xid, event) in xrr {
                    process(&event, opts);

                    unack.push(xid, &event)
                }

                unack.ack_all(&opts)
            }
            Err(e) => error!("Stream error {:?}", e),
        }
    }
}

fn process(event: &StreamInput, opts: &mut StreamOpts) {
    match &event {
        StreamInput::AB(ab) => {
            process_attach_bot(&ab, opts);
        }
        StreamInput::GS(game_state) => {
            process_game_state(&game_state, opts);
        }
    }
}

fn process_attach_bot(ab: &AttachBot, opts: &mut StreamOpts) {
    use bot_model::api::BotAttached;
    if let Err(e) = opts.attached_bots_repo.attach(&ab.game_id, ab.player) {
        error!("Error attaching bot {:?}", e)
    } else {
        info!("Stream: Set up game state for attach bot {:?}", ab);
        let mut game_state = move_model::GameState {
            game_id: core_model::GameId(ab.game_id.0),
            captures: move_model::Captures::default(),
            turn: 1,
            moves: vec![],
            board: move_model::Board::default(),
            player_up: move_model::Player::BLACK,
        };
        if let Some(bs) = ab.board_size {
            game_state.board.size = bs.into()
        }

        if let Err(e) = opts.xadd.xadd_game_state(&game_state) {
            error!(
                "Error writing redis stream for game state changelog : {:?}",
                e
            )
        } else if let Err(e) = opts.xadd.xadd_bot_attached(BotAttached {
            game_id: ab.game_id.clone(),
            player: ab.player,
        }) {
            error!("Error xadd bot attached {:?}", e)
        }

        if let Err(e) = opts.board_size_repo.put(&ab.game_id, game_state.board.size) {
            error!("Failed to write board size {:?}", e)
        }

        if let Err(e) = opts.difficulty_repo.put(&ab.game_id, ab.difficulty) {
            error!("Failed to put difficulty {:?}", e)
        }
    }
}

fn process_game_state(game_state: &GameState, opts: &mut StreamOpts) {
    let player_up = game_state.player_up;
    let game_id = &game_state.game_id;
    match (
        opts.attached_bots_repo.is_attached(&game_id, player_up),
        opts.difficulty_repo.get(&game_id),
    ) {
        (Ok(true), Ok(difficulty)) => {
            if let Err(e) = opts.compute_move_in.send(ComputeMove {
                game_id: game_id.clone(),
                game_state: game_state.clone(),
                max_visits: max_visits::convert(difficulty.unwrap_or(bot_model::Difficulty::Max)),
            }) {
                error!("WS SEND ERROR {:?}", e)
            }
        }
        (Ok(false), Ok(_)) => info!("Ignoring {:?} {:?}", game_id, player_up),
        (Err(e), Ok(_)) => error!("Game Repo is_attached {:?}", e),
        (Ok(_), Err(e)) => error!("Difficulty get {:?}", e),
        (Err(e), Err(f)) => error!("So many errors {:?} {:?}", e, f),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::*;
    use crate::stream::xadd::*;
    use bot_model::api::*;
    use bot_model::*;
    use core_model::*;
    use crossbeam_channel::Sender;
    use crossbeam_channel::{select, unbounded, Receiver};
    use move_model::*;
    use redis_streams::XReadEntryId;
    use std::sync::atomic::{AtomicU16, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    #[derive(Clone)]
    struct FakeAttachedBotsRepo {
        pub members: Arc<Mutex<Vec<(GameId, Player)>>>,
    }
    impl AttachedBotsRepo for FakeAttachedBotsRepo {
        fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr> {
            Ok(self
                .members
                .lock()
                .expect("lock")
                .contains(&(game_id.clone(), player)))
        }
        fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr> {
            Ok(self
                .members
                .lock()
                .expect("lock")
                .push((game_id.clone(), player)))
        }
    }

    static FAKE_BOARD_SIZE: AtomicU16 = AtomicU16::new(0);
    struct FakeBoardSizeRepo;
    impl BoardSizeRepo for FakeBoardSizeRepo {
        fn get(&self, _game_id: &GameId) -> Result<u16, RepoErr> {
            Ok(FAKE_BOARD_SIZE.load(Ordering::SeqCst))
        }
        fn put(&self, _game_id: &GameId, board_size: u16) -> Result<(), RepoErr> {
            FAKE_BOARD_SIZE.store(board_size, Ordering::SeqCst);
            Ok(())
        }
    }

    struct FakeDifficultyRepo;
    impl DifficultyRepo for FakeDifficultyRepo {
        fn get(&self, _game_id: &GameId) -> Result<Option<Difficulty>, RepoErr> {
            Ok(None)
        }

        fn put(&self, _game_id: &GameId, _difficulty: Difficulty) -> Result<(), RepoErr> {
            Ok(())
        }
    }

    struct FakeXAdder {
        added_in: Sender<move_model::GameState>,
    }
    impl xadd::XAdder for FakeXAdder {
        fn xadd_game_state(
            &self,
            game_state: &move_model::GameState,
        ) -> Result<(), StreamAddError> {
            Ok(self.added_in.send(game_state.clone()).expect("send add"))
        }
        fn xadd_bot_attached(
            &self,
            _bot_attached: BotAttached,
        ) -> Result<(), crate::stream::xadd::StreamAddError> {
            Ok(())
        }
        fn xadd_make_move_command(&self, _command: &MakeMove) -> Result<(), StreamAddError> {
            Ok(info!("Doing nothing for xadd make move"))
        }
    }

    struct FakeXReader {
        incoming_game_state: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
        init_data: Mutex<Vec<(XReadEntryId, StreamInput)>>,
    }
    impl xread::XReader for FakeXReader {
        fn xread_sorted(
            &self,
        ) -> Result<Vec<(redis_streams::XReadEntryId, StreamInput)>, xread::StreamReadError>
        {
            let mut v = vec![];

            if let Ok(mut the_start) = self.init_data.lock() {
                if !the_start.is_empty() {
                    v.extend(the_start.clone());
                    *the_start = vec![];
                }
            }

            v.extend(self.incoming_game_state.lock().expect("xrl").clone());

            let mut data = self.incoming_game_state.lock().expect("locked gs");
            *data = vec![];

            Ok(v)
        }
    }

    #[test]
    fn process_test() {
        let (compute_move_in, _): (Sender<ComputeMove>, _) = unbounded();
        let (added_in, added_out): (
            Sender<move_model::GameState>,
            Receiver<move_model::GameState>,
        ) = unbounded();

        let bots_attached = Arc::new(Mutex::new(vec![]));
        let attached_bots_repo = Box::new(FakeAttachedBotsRepo {
            members: bots_attached.clone(),
        });
        let abr = attached_bots_repo.clone();

        let board_size_repo = Arc::new(FakeBoardSizeRepo);

        let difficulty_repo = Box::new(FakeDifficultyRepo);

        const GAME_ID: GameId = GameId(Uuid::nil());
        let player = Player::WHITE;
        let board_size = Some(13);
        let incoming_game_state = Arc::new(Mutex::new(vec![]));
        let xreader = Box::new(FakeXReader {
            incoming_game_state: incoming_game_state.clone(),
            init_data: Mutex::new(vec![(
                XReadEntryId {
                    millis_time: 10,
                    seq_no: 0,
                },
                StreamInput::AB(AttachBot {
                    game_id: GAME_ID.clone(),
                    player,
                    board_size,
                    difficulty: Difficulty::Max,
                }),
            )]),
        });
        let xadder = Arc::new(FakeXAdder { added_in });
        struct FakeXAck {
            acked: Mutex<Vec<XReadEntryId>>,
        };
        impl crate::stream::xack::XAck for FakeXAck {
            fn ack_attach_bot(
                &self,
                xids: &[XReadEntryId],
            ) -> Result<(), super::xack::StreamAckError> {
                if let Ok(mut a) = self.acked.lock() {
                    a.extend(xids)
                }
                Ok(())
            }

            fn ack_game_states_changelog(
                &self,
                xids: &[XReadEntryId],
            ) -> Result<(), super::xack::StreamAckError> {
                if let Ok(mut a) = self.acked.lock() {
                    a.extend(xids)
                }
                Ok(())
            }
        }

        thread::spawn(move || {
            let mut opts = StreamOpts {
                compute_move_in,
                attached_bots_repo,
                difficulty_repo,
                board_size_repo,
                xread: xreader,
                xadd: xadder,
                xack: Arc::new(FakeXAck {
                    acked: Mutex::new(vec![]),
                }),
            };

            xread_loop(&mut opts)
        });

        // process xadd of game state correctly
        thread::spawn(move || loop {
            select! {
                recv(added_out) -> msg => if let Ok(a) = msg {
                    let mut data =  incoming_game_state.lock().expect("locked gs");
                    data.push(
                        (XReadEntryId{millis_time: 1, seq_no: 0},
                            StreamInput::GS(a)));
                }
            }
        });

        thread::sleep(Duration::from_millis(1));
        assert!(abr.is_attached(&GAME_ID, player).expect("ab repo"));
    }
}
