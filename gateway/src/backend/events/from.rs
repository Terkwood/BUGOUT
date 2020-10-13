use crate::backend::events as be;
use crate::model::{Coord, HistoryProvidedEvent, Move, MoveMadeEvent, Player, Visibility};
use color_model as color;
use lobby_model as lobby;
use move_model as moves;
use sync_model as sync;

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
impl From<color::api::ColorsChosen> for be::ColorsChosenEvent {
    fn from(_: color::api::ColorsChosen) -> Self {
        todo!()
    }
}

impl From<lobby::api::GameReady> for be::GameReadyBackendEvent {
    fn from(_: lobby::api::GameReady) -> Self {
        todo!()
    }
}

impl From<lobby::api::PrivateGameRejected> for be::PrivateGameRejectedBackendEvent {
    fn from(_: lobby::api::PrivateGameRejected) -> Self {
        todo!()
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
impl From<sync::move_model::Player> for Player {
    fn from(p: sync::move_model::Player) -> Self {
        match p {
            sync::move_model::Player::BLACK => Player::BLACK,
            sync::move_model::Player::WHITE => Player::WHITE,
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
impl From<sync::move_model::Coord> for Coord {
    fn from(c: sync::move_model::Coord) -> Self {
        Self { x: c.x, y: c.y }
    }
}

impl From<micro_model_moves::Coord> for Coord {
    fn from(c: micro_model_moves::Coord) -> Self {
        Self { x: c.x, y: c.y }
    }
}
