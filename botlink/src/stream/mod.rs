pub mod topics;
pub mod xread;

use crate::registry::Components;
use crate::repo::{AttachedBotsRepo, EntryIdRepo, EntryIdType};
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use micro_model_bot::gateway::AttachBot;
use micro_model_bot::{ComputeMove, MoveComputed};
use topics::Topics;
use xread::StreamData;

pub fn process(topics: Topics, opts: &mut StreamOpts) {
    info!("Processing {:#?}", topics);
    loop {
        match opts.entry_id_repo.fetch_all() {
            Ok(entry_ids) => match opts.xreader.xread_sorted(entry_ids, &topics) {
                Ok(xrr) => {
                    for time_ordered_event in xrr {
                        match time_ordered_event {
                            (entry_id, StreamData::AB(AttachBot { game_id, player })) => {
                                if let Err(e) = opts.attached_bots_repo.attach(&game_id, player) {
                                    error!("Error attaching bot {:?}", e)
                                } else {
                                    if let Err(e) = opts
                                        .entry_id_repo
                                        .update(EntryIdType::AttachBotEvent, entry_id)
                                    {
                                        error!("Error saving entry ID for attach bot {:?}", e)
                                    }
                                }
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
    pub xreader: Box<dyn xread::XReader>,
    pub compute_move_in: Sender<ComputeMove>,
    pub move_computed_out: Receiver<MoveComputed>,
}

impl StreamOpts {
    pub fn from(components: Components) -> Self {
        StreamOpts {
            attached_bots_repo: components.game_repo,
            entry_id_repo: components.entry_id_repo,
            xreader: components.xreader,
            compute_move_in: components.compute_move_in,
            move_computed_out: components.move_computed_out,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::*;
    use crossbeam_channel::unbounded;
    use micro_model_moves::*;
    use redis_streams::XReadEntryId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use uuid::Uuid;

    struct FakeEntryIdRepo;
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
        ) -> Result<(), redis_conn_pool::redis::RedisError> {
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

    struct FakeAttachedBotsRepo {
        pub members: Vec<(GameId, Player)>,
    }
    impl AttachedBotsRepo for FakeAttachedBotsRepo {
        fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr> {
            Ok(self.members.contains(&(game_id.clone(), player)))
        }
        fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr> {
            Ok(self.members.push((game_id.clone(), player)))
        }
    }

    struct FakeXReader;
    impl xread::XReader for FakeXReader {
        fn xread_sorted(
            &self,
            entry_ids: AllEntryIds,
            _: &Topics,
        ) -> Result<
            Vec<(redis_streams::XReadEntryId, StreamData)>,
            redis_conn_pool::redis::RedisError,
        > {
            let game_id = GameId(Uuid::nil());
            let player = Player::WHITE;
            Ok(vec![(
                XReadEntryId::default(),
                StreamData::AB(AttachBot { game_id, player }),
            )]
            .iter()
            .filter(|(eid, data)| {
                eid < match data {
                    StreamData::AB(_) => &entry_ids.attach_bot_eid,
                    StreamData::GS(_, _) => &entry_ids.game_states_eid,
                }
            })
            .cloned()
            .collect())
        }
    }

    #[test]
    fn basic_test() {
        let (compute_move_in, _): (Sender<ComputeMove>, _) = unbounded();
        let (_, move_computed_out): (_, Receiver<MoveComputed>) = unbounded();

        thread::spawn(move || {
            let entry_id_repo = Box::new(FakeEntryIdRepo);
            let attached_bots_repo = Box::new(FakeAttachedBotsRepo { members: vec![] });
            let xreader = Box::new(FakeXReader);

            let mut opts = StreamOpts {
                compute_move_in,
                move_computed_out,
                entry_id_repo,
                attached_bots_repo,
                xreader,
            };

            process(Topics::default(), &mut opts)
        });
        todo!()
    }
}
