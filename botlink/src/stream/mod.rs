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
    use crossbeam_channel::unbounded;
    use std::thread;
    use std::time::Duration;
    #[test]
    fn basic_test() {
        let (compute_move_in, _): (Sender<ComputeMove>, _) = unbounded();
        let (_, move_computed_out): (_, Receiver<MoveComputed>) = unbounded();

        thread::spawn(move || {
            let entry_id_repo = todo!();
            let attached_bots_repo = todo!();
            let xreader = todo!();

            let opts = StreamOpts {
                compute_move_in,
                move_computed_out,
                entry_id_repo,
                attached_bots_repo,
                xreader,
            };

            process(todo!(), &mut opts)
        });
        todo!()
    }
}
