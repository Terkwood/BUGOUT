mod convert;
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

use convert::Convert;
pub use input::StreamInput;
use log::{error, info};
use micro_model_bot::gateway::AttachBot;
use micro_model_bot::ComputeMove;

pub fn xread_loop(opts: &mut StreamOpts) {
    let mut unack = Unacknowledged::default();
    loop {
        match opts.xreader.xread_sorted() {
            Ok(xrr) => {
                for (xid, event) in xrr {
                    process(&event, opts);

                    unack.push(xid, &event)
                }

                unack.ack_all()
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
            let player_up = game_state.player_up.convert();
            let game_id = game_state.game_id.convert();
            match opts.attached_bots_repo.is_attached(&game_id, player_up) {
                Ok(bot_game) => {
                    if bot_game {
                        let convert_state = game_state.convert();
                        if let Err(e) = opts.compute_move_in.send(ComputeMove {
                            game_id,
                            game_state: convert_state,
                        }) {
                            error!("WS SEND ERROR {:?}", e)
                        }
                    } else {
                        info!("Ignoring {:?} {:?}", game_id, player_up)
                    };
                }
                Err(e) => error!("Game Repo error is_attached {:?}", e),
            }
        }
    }
}

fn process_attach_bot(ab: &AttachBot, opts: &mut StreamOpts) {
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

        if let Err(e) = opts.xadder.xadd_game_state(&game_state) {
            error!(
                "Error writing redis stream for game state changelog : {:?}",
                e
            )
        } else if let Err(e) =
            opts.xadder
                .xadd_bot_attached(micro_model_bot::gateway::BotAttached {
                    game_id: ab.game_id.clone(),
                    player: ab.player,
                })
        {
            error!("Error xadd bot attached {:?}", e)
        }

        if let Err(e) = opts
            .board_size_repo
            .set_board_size(&ab.game_id, game_state.board.size)
        {
            error!("Failed to write board size {:?}", e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::*;
    use crate::stream::xadd::*;
    use crossbeam_channel::Sender;
    use crossbeam_channel::{select, unbounded, Receiver};
    use micro_model_moves::*;
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
        fn get_board_size(&self, _game_id: &GameId) -> Result<u16, RepoErr> {
            Ok(FAKE_BOARD_SIZE.load(Ordering::SeqCst))
        }
        fn set_board_size(&self, _game_id: &GameId, board_size: u16) -> Result<(), RepoErr> {
            FAKE_BOARD_SIZE.store(board_size, Ordering::SeqCst);
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
            _bot_attached: micro_model_bot::gateway::BotAttached,
        ) -> Result<(), crate::stream::xadd::StreamAddError> {
            Ok(())
        }
        fn xadd_make_move_command(&self, _command: &MakeMoveCommand) -> Result<(), StreamAddError> {
            Ok(info!("Doing nothing for xadd make move"))
        }
    }

    struct FakeXReader {
        game_id: GameId,
        player: Player,
        board_size: Option<u8>,
        incoming_game_state: Arc<Mutex<Option<(XReadEntryId, StreamInput)>>>,
    }
    impl xread::XReader for FakeXReader {
        fn xread_sorted(
            &self,
        ) -> Result<Vec<(redis_streams::XReadEntryId, StreamInput)>, redis::RedisError> {
            let game_id = self.game_id.clone();
            let player = self.player;
            let board_size = self.board_size;
            let mut v: Vec<(XReadEntryId, StreamInput)> = vec![(
                XReadEntryId {
                    millis_time: 10,
                    seq_no: 0,
                },
                StreamInput::AB(AttachBot {
                    game_id,
                    player,
                    board_size,
                }),
            )];
            if let Some((inc_eid, inc_game_state)) =
                self.incoming_game_state.lock().expect("xrl").clone()
            {
                v.push((inc_eid, inc_game_state));
            }

            Ok(todo!())
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

        const GAME_ID: GameId = GameId(Uuid::nil());
        let player = Player::WHITE;
        let board_size = Some(13);
        let incoming_game_state = Arc::new(Mutex::new(None));
        let xreader = Box::new(FakeXReader {
            game_id: GAME_ID.clone(),
            player,
            board_size,
            incoming_game_state: incoming_game_state.clone(),
        });
        let xadder = Arc::new(FakeXAdder { added_in });

        thread::spawn(move || {
            let mut opts = StreamOpts {
                compute_move_in,
                attached_bots_repo,
                board_size_repo,
                xreader,
                xadder,
            };

            xread_loop(&mut opts)
        });

        // process xadd of game state correctly
        thread::spawn(move || loop {
            select! {
                recv(added_out) -> msg => if let Ok(a) = msg {
                    let mut data =  incoming_game_state.lock().expect("locked gs");
                    *data = Some((XReadEntryId{millis_time: 1, seq_no: 0}, StreamInput::GS(a))); }
            }
        });

        thread::sleep(Duration::from_millis(1));
        assert!(abr.is_attached(&GAME_ID, player).expect("ab repo"));

        let timeoutdur = Some(Duration::from_millis(30));
    }
}
