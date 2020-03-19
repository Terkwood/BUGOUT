pub mod topics;
pub mod xread;

use crate::registry::Components;
use crate::repo::entry_id::EntryIdType;
use crate::repo::{EntryIdRepo, GameRepo};
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
                                if let Err(e) = opts.game_repo.attach(game_id, player) {
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
                            (_entry_id, StreamData::GS(_game_id, _game_state)) => todo!(),
                        }
                    }
                }
                _ => todo!(),
            },
            Err(e) => error!("Redis err in xread: {:#?}", e),
        }
    }
}

pub struct StreamOpts {
    pub game_repo: Box<dyn GameRepo>,
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub xreader: Box<dyn xread::XReader>,
    pub compute_move_out: Receiver<ComputeMove>,
    pub move_computed_in: Sender<MoveComputed>,
}

impl StreamOpts {
    pub fn from(components: Components) -> Self {
        StreamOpts {
            game_repo: components.game_repo,
            entry_id_repo: components.entry_id_repo,
            xreader: components.xreader,
            compute_move_out: components.compute_move_out,
            move_computed_in: components.move_computed_in,
        }
    }
}
