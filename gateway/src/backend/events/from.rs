use crate::backend::events as be;
use crate::model::{Coord, HistoryProvidedEvent, Move, MoveMadeEvent, Player, Visibility};
use color_model as color;
use lobby_model as lobby;
use move_model as moves;
use sync_model as sync;

impl From<moves::MoveMade> for MoveMadeEvent {
    fn from(m: moves::MoveMade) -> Self {
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
    fn from(c: color::api::ColorsChosen) -> Self {
        Self {
            game_id: c.game_id.0,
            black: c.black.0,
            white: c.white.0,
        }
    }
}

impl From<lobby::api::GameReady> for be::GameReadyBackendEvent {
    fn from(g: lobby::api::GameReady) -> Self {
        Self {
            game_id: g.game_id.0,
            board_size: g.board_size as u8,
            event_id: g.event_id.0,
            sessions: crate::model::GameSessions {
                first: g.sessions.0.0,
                second: g.sessions.1.0,
            },
        }
    }
}

impl From<lobby::api::PrivateGameRejected> for be::PrivateGameRejectedBackendEvent {
    fn from(p: lobby::api::PrivateGameRejected) -> Self {
        Self {
            game_id: p.game_id.0,
            session_id: p.session_id.0,
            event_id: p.event_id.0,
            client_id: p.client_id.0
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
  
impl From<moves::Coord> for Coord {
    fn from(c: moves::Coord) -> Self {
        Self { x: c.x, y: c.y }
    }
}
  