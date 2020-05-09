pub mod topics;
mod write_moves;
pub mod xadd;
pub mod xread;

use crate::registry::Components;
use crate::repo::{AttachedBotsRepo, BoardSizeRepo, EntryIdRepo, EntryIdType};
use crossbeam_channel::Sender;
use log::{error, info};
use micro_model_bot::gateway::AttachBot;
use micro_model_bot::ComputeMove;
use micro_model_moves::GameState;
use redis_streams::XReadEntryId;
use std::sync::Arc;
pub use write_moves::write_moves;
use xread::StreamData;

pub fn process(opts: &mut StreamOpts) {
    loop {
        match opts.entry_id_repo.fetch_all() {
            Ok(entry_ids) => match opts.xreader.xread_sorted(entry_ids) {
                Ok(xrr) => {
                    for time_ordered_event in xrr {
                        match time_ordered_event {
                            (entry_id, StreamData::AB(ab)) => {
                                process_attach_bot(ab, entry_id, opts)
                            }
                            (entry_id, StreamData::GS(game_id, game_state)) => {
                                match opts
                                    .attached_bots_repo
                                    .is_attached(&game_id, game_state.player_up)
                                {
                                    Ok(bot_game) => {
                                        if bot_game {
                                            if let Err(e) = opts.compute_move_in.send(ComputeMove {
                                                game_id,
                                                game_state,
                                            }) {
                                                error!("WS SEND ERROR {:?}", e)
                                            }
                                        } else {
                                            info!(
                                                "Ignoring {:?} {:?}",
                                                game_id, game_state.player_up
                                            )
                                        };
                                        if let Err(e) = opts
                                            .entry_id_repo
                                            .update(EntryIdType::GameStateChangelog, entry_id)
                                        {
                                            error!("Failed to save entry ID for game state {:?}", e)
                                        }
                                    }
                                    Err(e) => error!("Game Repo error is_attached {:?}", e),
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("Stream error {:?}", e),
            },
            Err(e) => error!("Redis err in xread: {:#?}", e),
        }
    }
}

pub struct StreamOpts {
    pub attached_bots_repo: Box<dyn AttachedBotsRepo>,
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub board_size_repo: Arc<dyn BoardSizeRepo>,
    pub xreader: Box<dyn xread::XReader>,
    pub xadder: Arc<dyn xadd::XAdder>,
    pub compute_move_in: Sender<ComputeMove>,
}

impl StreamOpts {
    pub fn from(components: Components) -> Self {
        StreamOpts {
            attached_bots_repo: components.ab_repo,
            entry_id_repo: components.entry_id_repo,
            board_size_repo: components.board_size_repo,
            xreader: components.xreader,
            xadder: components.xadder,
            compute_move_in: components.compute_move_in,
        }
    }
}

fn process_attach_bot(ab: AttachBot, entry_id: XReadEntryId, opts: &mut StreamOpts) {
    if let Err(e) = opts.attached_bots_repo.attach(&ab.game_id, ab.player) {
        error!("Error attaching bot {:?}", e)
    } else if let Err(e) = opts
        .entry_id_repo
        .update(EntryIdType::AttachBotEvent, entry_id)
    {
        error!("Error saving entry ID for attach bot {:?}", e)
    } else {
        let mut game_state = GameState::default();
        if let Some(bs) = ab.board_size {
            game_state.board.size = bs.into()
        }

        if let Err(e) = opts.xadder.xadd_game_state(&ab.game_id, &game_state) {
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
    use crossbeam_channel::{after, never, select, unbounded, Receiver};
    use micro_model_moves::*;
    use redis_streams::XReadEntryId;
    use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    #[derive(Clone)]
    struct FakeEntryIdRepo {
        eid_update_in: Sender<(EntryIdType, XReadEntryId)>,
    }
    static FAKE_AB_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_AB_SEQNO: AtomicU64 = AtomicU64::new(0);
    static FAKE_GS_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_GS_SEQNO: AtomicU64 = AtomicU64::new(0);
    impl EntryIdRepo for FakeEntryIdRepo {
        fn fetch_all(&self) -> Result<AllEntryIds, RepoErr> {
            Ok(AllEntryIds {
                attach_bot_eid: XReadEntryId {
                    millis_time: FAKE_AB_MILLIS.load(Ordering::SeqCst),
                    seq_no: FAKE_AB_SEQNO.load(Ordering::SeqCst),
                },
                game_states_eid: XReadEntryId {
                    millis_time: FAKE_GS_MILLIS.load(Ordering::SeqCst),
                    seq_no: FAKE_GS_MILLIS.load(Ordering::SeqCst),
                },
            })
        }
        fn update(
            &self,
            entry_id_type: EntryIdType,
            entry_id: redis_streams::XReadEntryId,
        ) -> Result<(), RepoErr> {
            self.eid_update_in
                .send((entry_id_type.clone(), entry_id))
                .expect("eid update send");
            Ok(match entry_id_type {
                EntryIdType::AttachBotEvent => {
                    FAKE_AB_MILLIS.store(entry_id.millis_time, Ordering::SeqCst);
                    FAKE_AB_SEQNO.store(entry_id.seq_no, Ordering::SeqCst)
                }
                EntryIdType::GameStateChangelog => {
                    FAKE_GS_MILLIS.store(entry_id.millis_time, Ordering::SeqCst);
                    FAKE_GS_SEQNO.store(entry_id.seq_no, Ordering::SeqCst)
                }
            })
        }
    }

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
        added_in: Sender<(GameId, GameState)>,
    }
    impl xadd::XAdder for FakeXAdder {
        fn xadd_game_state(
            &self,
            game_id: &GameId,
            game_state: &GameState,
        ) -> Result<(), XAddError> {
            Ok(self
                .added_in
                .send((game_id.clone(), game_state.clone()))
                .expect("send add"))
        }
        fn xadd_bot_attached(
            &self,
            _bot_attached: micro_model_bot::gateway::BotAttached,
        ) -> Result<(), crate::stream::xadd::XAddError> {
            Ok(())
        }
        fn xadd_make_move_command(&self, _command: &MakeMoveCommand) -> Result<(), XAddError> {
            Ok(info!("Doing nothing for xadd make move"))
        }
    }

    struct FakeXReader {
        game_id: GameId,
        player: Player,
        board_size: Option<u8>,
        incoming_game_state: Arc<Mutex<Option<(XReadEntryId, StreamData)>>>,
    }
    impl xread::XReader for FakeXReader {
        fn xread_sorted(
            &self,
            entry_ids: AllEntryIds,
        ) -> Result<
            Vec<(redis_streams::XReadEntryId, StreamData)>,
            redis_conn_pool::redis::RedisError,
        > {
            let game_id = self.game_id.clone();
            let player = self.player;
            let board_size = self.board_size;
            let mut v: Vec<(XReadEntryId, StreamData)> = vec![(
                XReadEntryId {
                    millis_time: 10,
                    seq_no: 0,
                },
                StreamData::AB(AttachBot {
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

            Ok(v.iter()
                .filter(|(eid, data)| {
                    eid > match data {
                        StreamData::AB(_) => &entry_ids.attach_bot_eid,
                        StreamData::GS(_, _) => &entry_ids.game_states_eid,
                    }
                })
                .cloned()
                .collect())
        }
    }

    #[test]
    fn process_test() {
        let (compute_move_in, _): (Sender<ComputeMove>, _) = unbounded();
        let (eid_update_in, eid_update_out) = unbounded();
        let (added_in, added_out): (Sender<(GameId, GameState)>, Receiver<(GameId, GameState)>) =
            unbounded();

        let entry_id_repo = Box::new(FakeEntryIdRepo { eid_update_in });

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
                entry_id_repo,
                attached_bots_repo,
                board_size_repo,
                xreader,
                xadder,
            };

            process(&mut opts)
        });

        // process xadd of game state correctly
        thread::spawn(move || loop {
            select! {
                recv(added_out) -> msg => if let Ok(a) = msg {
                    let mut data =  incoming_game_state.lock().expect("locked gs");
                    *data = Some((XReadEntryId{millis_time: 1, seq_no: 0}, StreamData::GS(a.0, a.1))); }
            }
        });

        thread::sleep(Duration::from_millis(1));
        assert!(abr.is_attached(&GAME_ID, player).expect("ab repo"));

        let timeoutdur = Some(Duration::from_millis(30));

        // Create a channel that times out after the specified duration.
        let timeout = timeoutdur.map(|d| after(d)).unwrap_or(never());
        let mut eid_updates_observed = vec![];
        select! {
            recv(eid_update_out) -> msg => eid_updates_observed.push(msg.expect("msg")),
            recv(timeout) -> _ => panic!("unexpected timeout")
        }
        select! {
            recv(eid_update_out) -> msg => eid_updates_observed.push(msg.expect("msg")),
            recv(timeout) -> _ => panic!("unexpected timeout 2")
        }

        assert!(eid_updates_observed.len() == 2);
        assert_eq!(eid_updates_observed[0].0, EntryIdType::AttachBotEvent);
        assert!(eid_updates_observed[0].1 > XReadEntryId::default());
        assert_eq!(eid_updates_observed[1].0, EntryIdType::GameStateChangelog);
        assert!(eid_updates_observed[1].1 > XReadEntryId::default());
    }
}
