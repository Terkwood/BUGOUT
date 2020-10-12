use super::entry_id_repo::*;
use super::xread::XReader;
use crate::backend_events::BackendEvents;
use crate::model::{Coord, MoveMadeEvent, Player};
use color_model as color;
use crossbeam_channel::Sender;
use lobby_model as lobby;
use log::error;
use sync_model as sync;

#[derive(Clone, Debug)]
pub enum StreamData {
    BotAttached(micro_model_bot::gateway::BotAttached),
    MoveMade(micro_model_moves::MoveMade),
    HistoryProvided(sync::api::HistoryProvided),
    SyncReply(sync::api::SyncReply),
    WaitForOpponent(lobby::api::WaitForOpponent),
    GameReady(lobby::api::GameReady),
    ColorsChosen(color::api::ColorsChosen),
}

pub fn process(events_in: Sender<BackendEvents>, opts: StreamOpts) {
    loop {
        match opts.entry_id_repo.fetch_all() {
            Err(e) => error!("cannot fetch entry id repo {:?}", e),
            Ok(entry_ids) => match opts.xreader.xread_sorted(entry_ids) {
                Err(e) => error!("cannot xread {:?}", e),
                Ok(xrr) => {
                    for event in xrr {
                        match event {
                            (xid, StreamData::BotAttached(b)) => {
                                if let Err(e) = events_in.send(BackendEvents::BotAttached(b)) {
                                    error!("send err bot attached {:?}", e)
                                } else if let Err(e) =
                                    opts.entry_id_repo.update(EntryIdType::BotAttached, xid)
                                {
                                    error!("err tracking EID bot attached {:?}", e)
                                }
                            }
                            (
                                xid,
                                StreamData::MoveMade(micro_model_moves::MoveMade {
                                    game_id,
                                    coord,
                                    reply_to,
                                    player,
                                    captured,
                                    event_id,
                                }),
                            ) => {
                                if let Err(e) =
                                    events_in.send(BackendEvents::MoveMade(MoveMadeEvent {
                                        game_id: game_id.0,
                                        coord: coord.map(|c| Coord { x: c.x, y: c.y }),
                                        reply_to: reply_to.0,
                                        player: match player {
                                            micro_model_moves::Player::BLACK => Player::BLACK,
                                            _ => Player::WHITE,
                                        },
                                        captured: captured
                                            .iter()
                                            .map(|c| Coord { x: c.x, y: c.y })
                                            .collect(),
                                        event_id: event_id.0,
                                    }))
                                {
                                    error!("send err move made {:?}", e)
                                }

                                if let Err(e) =
                                    opts.entry_id_repo.update(EntryIdType::MoveMade, xid)
                                {
                                    error!("err tracking EID move made {:?}", e)
                                }
                            }
                            (_, StreamData::HistoryProvided(_)) => todo!(),
                            (_, StreamData::SyncReply(_)) => todo!(),
                            (_, StreamData::WaitForOpponent(_)) => todo!(),
                            (_, StreamData::GameReady(_)) => todo!(),
                            (_, StreamData::ColorsChosen(_)) => todo!(),
                        }
                    }
                }
            },
        }
    }
}

pub struct StreamOpts {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub xreader: Box<dyn XReader>,
}
