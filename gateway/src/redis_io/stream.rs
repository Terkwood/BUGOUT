use super::entry_id_repo::*;
use super::xread::XReader;
use crate::backend_events as be;
use crate::backend_events::BackendEvents;
use crate::model::{Coord, HistoryProvidedEvent, Move, MoveMadeEvent, Player, Visibility};
use color_model as color;
use crossbeam_channel::Sender;
use lobby_model as lobby;
use log::error;
use move_model as moves;
use redis_streams::XReadEntryId;
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
                    for (xid, data) in xrr {
                        process_event(xid, data, &events_in, &opts)
                    }
                }
            },
        }
    }
}

fn process_event(
    xid: XReadEntryId,
    data: StreamData,
    events_in: &Sender<BackendEvents>,
    opts: &StreamOpts,
) {
    match data {
        StreamData::BotAttached(b) => {
            if let Err(e) = events_in.send(BackendEvents::BotAttached(b)) {
                error!("send err bot attached {:?}", e)
            } else if let Err(e) = opts.entry_id_repo.update(EntryIdType::BotAttached, xid) {
                error!("err tracking EID bot attached {:?}", e)
            }
        }
        StreamData::MoveMade(m) => {
            if let Err(e) = events_in.send(BackendEvents::from(StreamData::MoveMade(m))) {
                error!("send err move made {:?}", e)
            }

            if let Err(e) = opts.entry_id_repo.update(EntryIdType::MoveMade, xid) {
                error!("err tracking EID move made {:?}", e)
            }
        }
        StreamData::HistoryProvided(_) => todo!(),
        StreamData::SyncReply(_) => todo!(),
        StreamData::WaitForOpponent(_) => todo!(),
        StreamData::GameReady(_) => todo!(),
        StreamData::PrivGameRejected(_) => todo!(),
        StreamData::ColorsChosen(_) => todo!(),
    }
}

pub struct StreamOpts {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub xreader: Box<dyn XReader>,
}

impl From<StreamData> for BackendEvents {
    fn from(stream_data: StreamData) -> Self {
        match stream_data {
            StreamData::MoveMade(m) => BackendEvents::MoveMade(MoveMadeEvent::from(m)),
            StreamData::BotAttached(b) => BackendEvents::BotAttached(b),
            StreamData::HistoryProvided(h) => {
                BackendEvents::HistoryProvided(HistoryProvidedEvent::from(h))
            }
            StreamData::SyncReply(s) => {
                BackendEvents::SyncReply(be::SyncReplyBackendEvent::from(s))
            }
            StreamData::WaitForOpponent(w) => {
                be::BackendEvents::WaitForOpponent(be::WaitForOpponentBackendEvent::from(w))
            }
            StreamData::GameReady(g) => todo!(),
            StreamData::PrivGameRejected(p) => todo!(),
            StreamData::ColorsChosen(c) => todo!(),
        }
    }
}
impl From<micro_model_moves::MoveMade> for MoveMadeEvent {
    fn from(m: micro_model_moves::MoveMade) -> Self {
        MoveMadeEvent {
            game_id: m.game_id.0,
            coord: m.coord.map(|c| Coord::from(c)),
            reply_to: m.reply_to.0,
            player: Player::from(m.player),
            captured: m.captured.iter().map(|c| Coord::from(c.clone())).collect(),
            event_id: m.event_id.0,
        }
    }
}
impl From<sync::api::SyncReply> for be::SyncReplyBackendEvent {
    fn from(s: sync::api::SyncReply) -> Self {
        be::SyncReplyBackendEvent {
            game_id: s.game_id.0,
            reply_to: s.reply_to.0,
            session_id: s.session_id.0,
            turn: s.turn,
            player_up: Player::from(s.player_up),
            moves: s.moves.iter().map(|m| Move::from(m.clone())).collect(),
        }
    }
}
impl From<sync::api::HistoryProvided> for HistoryProvidedEvent {
    fn from(h: sync::api::HistoryProvided) -> Self {
        HistoryProvidedEvent {
            game_id: h.game_id.0,
            reply_to: h.reply_to.0,
            moves: h.moves.iter().map(|m| Move::from(m.clone())).collect(),
            event_id: h.event_id.0,
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
            player: Player::from(m.player),
            coord: m.coord.map(|c| Coord::from(c)),
        }
    }
}
impl From<moves::Player> for Player {
    fn from(p: moves::Player) -> Self {
        match p {
            moves::Player::BLACK => Player::BLACK,
            moves::Player::WHITE => Player::WHITE,
        }
    }
}
impl From<micro_model_moves::Player> for Player {
    fn from(p: micro_model_moves::Player) -> Self {
        match p {
            micro_model_moves::Player::BLACK => Player::BLACK,
            micro_model_moves::Player::WHITE => Player::WHITE,
        }
    }
}
impl From<moves::Coord> for Coord {
    fn from(c: moves::Coord) -> Self {
        Self { x: c.x, y: c.y }
    }
}

impl From<micro_model_moves::Coord> for Coord {
    fn from(c: micro_model_moves::Coord) -> Self {
        Self { x: c.x, y: c.y }
    }
}
