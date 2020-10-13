use super::entry_id_repo::*;
use super::xread::XReader;
use crate::backend_events as be;
use crate::backend_events::BackendEvents;
use crate::model::{Coord, Move, MoveMadeEvent, Player, Visibility};
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
    PrivGameRejected(lobby::api::PrivateGameRejected),
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
                            (xid, StreamData::MoveMade(m)) => {
                                if let Err(e) =
                                    events_in.send(BackendEvents::from(StreamData::MoveMade(m)))
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
                            (_, StreamData::PrivGameRejected(_)) => todo!(),
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

impl From<StreamData> for BackendEvents {
    fn from(stream_data: StreamData) -> Self {
        match stream_data {
            StreamData::MoveMade(micro_model_moves::MoveMade {
                game_id,
                coord,
                reply_to,
                player,
                captured,
                event_id,
            }) => BackendEvents::MoveMade(MoveMadeEvent {
                game_id: game_id.0,
                coord: coord.map(|c| Coord { x: c.x, y: c.y }),
                reply_to: reply_to.0,
                player: match player {
                    micro_model_moves::Player::BLACK => Player::BLACK,
                    _ => Player::WHITE,
                },
                captured: captured.iter().map(|c| Coord { x: c.x, y: c.y }).collect(),
                event_id: event_id.0,
            }),
            StreamData::BotAttached(b) => BackendEvents::BotAttached(b),
            StreamData::HistoryProvided(h) => {
                BackendEvents::HistoryProvided(crate::model::HistoryProvidedEvent {
                    game_id: h.game_id.0,
                    reply_to: h.reply_to.0,
                    moves: todo!(),
                    event_id: h.event_id.0,
                })
            }
            StreamData::SyncReply(s) => BackendEvents::SyncReply(be::SyncReplyBackendEvent {
                game_id: s.game_id.0,
                reply_to: s.reply_to.0,
                session_id: s.session_id.0,
                turn: s.turn,
                player_up: todo!(),
                moves: s.moves.iter().map(|m| Move::from(m.clone())).collect(),
            }),
            StreamData::WaitForOpponent(w) => {
                be::BackendEvents::WaitForOpponent(be::WaitForOpponentBackendEvent::from(w))
            }
            StreamData::GameReady(g) => todo!(),
            StreamData::PrivGameRejected(p) => todo!(),
            StreamData::ColorsChosen(c) => todo!(),
        }
    }
}

impl From<lobby::api::WaitForOpponent> for be::WaitForOpponentBackendEvent {
    fn from(w: lobby::api::WaitForOpponent) -> Self {
        Self {
            game_id: w.game_id.0,
            session_id: w.session_id.0,
            event_id: w.event_id.0,
            visibility: Visibility::from(w.visibility),
        }
    }
}
impl From<lobby::Visibility> for Visibility {
    fn from(v: lobby::Visibility) -> Self {
        match v {
            lobby::Visibility::Private => Visibility::Private,
            lobby::Visibility::Public => Visibility::Public,
        }
    }
}
impl From<sync::Move> for Move {
    fn from(m: sync::Move) -> Self {
        Self {
            turn: m.turn as i32,
            player: todo!("from"),
            coord: todo!("from"),
        }
    }
}
