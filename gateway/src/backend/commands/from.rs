use crate::backend::commands::*;
use crate::model::{ColorPref, Coord, Move, Player, ProvideHistoryCommand, Visibility};

use color_model as color;
use core_model as core;
use lobby_model as lobby;
use move_model as moves;
use sync_model as sync;

pub trait IntoShared<T> {
    fn into_shared(&self) -> T;
}

impl IntoShared<sync::api::ProvideHistory> for ProvideHistoryCommand {
    fn into_shared(&self) -> sync::api::ProvideHistory {
        sync::api::ProvideHistory {
            game_id: self.game_id.into_shared(),
            req_id: self.req_id.into_shared(),
        }
    }
}

impl IntoShared<core::GameId> for uuid::Uuid {
    fn into_shared(&self) -> core::GameId {
        core::GameId(self.clone())
    }
}

impl IntoShared<core::ReqId> for uuid::Uuid {
    fn into_shared(&self) -> core::ReqId {
        core::ReqId(self.clone())
    }
}

impl IntoShared<core::EventId> for uuid::Uuid {
    fn into_shared(&self) -> core::EventId {
        core::EventId(self.clone())
    }
}

impl IntoShared<core::SessionId> for uuid::Uuid {
    fn into_shared(&self) -> core::SessionId {
        core::SessionId(self.clone())
    }
}

impl IntoShared<core::ClientId> for uuid::Uuid {
    fn into_shared(&self) -> core::ClientId {
        core::ClientId(self.clone())
    }
}

impl IntoShared<sync::core_model::GameId> for uuid::Uuid {
    fn into_shared(&self) -> sync::core_model::GameId {
        sync::core_model::GameId(self.clone())
    }
}

impl IntoShared<sync::core_model::ReqId> for uuid::Uuid {
    fn into_shared(&self) -> sync::core_model::ReqId {
        sync::core_model::ReqId(self.clone())
    }
}

impl IntoShared<sync::core_model::EventId> for uuid::Uuid {
    fn into_shared(&self) -> sync::core_model::EventId {
        sync::core_model::EventId(self.clone())
    }
}

impl IntoShared<sync::core_model::SessionId> for uuid::Uuid {
    fn into_shared(&self) -> sync::core_model::SessionId {
        sync::core_model::SessionId(self.clone())
    }
}

impl IntoShared<sync::core_model::ClientId> for uuid::Uuid {
    fn into_shared(&self) -> sync::core_model::ClientId {
        sync::core_model::ClientId(self.clone())
    }
}

impl IntoShared<moves::Player> for crate::model::Player {
    fn into_shared(&self) -> moves::Player {
        match self {
            Player::BLACK => moves::Player::BLACK,
            Player::WHITE => moves::Player::WHITE,
        }
    }
}
impl IntoShared<sync::move_model::Player> for crate::model::Player {
    fn into_shared(&self) -> sync::move_model::Player {
        match self {
            Player::BLACK => sync::move_model::Player::BLACK,
            Player::WHITE => sync::move_model::Player::WHITE,
        }
    }
}

impl IntoShared<moves::Coord> for Coord {
    fn into_shared(&self) -> moves::Coord {
        moves::Coord {
            x: self.x,
            y: self.y,
        }
    }
}

impl IntoShared<sync::move_model::Coord> for Coord {
    fn into_shared(&self) -> sync::move_model::Coord {
        sync::move_model::Coord {
            x: self.x,
            y: self.y,
        }
    }
}

impl IntoShared<sync::Move> for Move {
    fn into_shared(&self) -> sync::Move {
        sync::Move {
            player: self.player.into_shared(),
            turn: self.turn as u32,
            coord: self.coord.map(|c| c.into_shared()),
        }
    }
}

impl IntoShared<sync::api::ReqSync> for ReqSyncBackendCommand {
    fn into_shared(&self) -> sync::api::ReqSync {
        sync::api::ReqSync {
            session_id: self.session_id.into_shared(),
            req_id: self.req_id.into_shared(),
            game_id: self.game_id.into_shared(),
            player_up: self.player_up.into_shared(),
            last_move: self.last_move.map(|m| m.into_shared()),
            turn: self.turn,
        }
    }
}

impl IntoShared<lobby::api::JoinPrivateGame> for JoinPrivateGameBackendCommand {
    fn into_shared(&self) -> lobby::api::JoinPrivateGame {
        lobby::api::JoinPrivateGame {
            game_id: self.game_id.into_shared(),
            client_id: self.client_id.into_shared(),
            session_id: self.session_id.into_shared(),
        }
    }
}

impl IntoShared<lobby::api::FindPublicGame> for FindPublicGameBackendCommand {
    fn into_shared(&self) -> lobby::api::FindPublicGame {
        lobby::api::FindPublicGame {
            client_id: self.client_id.into_shared(),
            session_id: self.session_id.into_shared(),
        }
    }
}
impl IntoShared<lobby::Visibility> for Visibility {
    fn into_shared(&self) -> lobby::Visibility {
        match self {
            Visibility::Private => lobby::Visibility::Private,
            Visibility::Public => lobby::Visibility::Public,
        }
    }
}

impl IntoShared<lobby::api::CreateGame> for CreateGameBackendCommand {
    fn into_shared(&self) -> lobby::api::CreateGame {
        lobby::api::CreateGame {
            client_id: self.client_id.into_shared(),
            game_id: None, // TODO think think about it 🤔,
            visibility: self.visibility.into_shared(),
            session_id: self.session_id.into_shared(),
            board_size: self.board_size,
        }
    }
}

impl From<ChooseColorPrefBackendCommand> for color::api::ChooseColorPref {
    fn from(c: ChooseColorPrefBackendCommand) -> Self {
        color::api::ChooseColorPref {
            client_id: c.client_id.into_shared(),
            color_pref: c.color_pref.into(),
            session_id: c.session_id.into_shared(),
        }
    }
}
impl From<ColorPref> for color::ColorPref {
    fn from(c: ColorPref) -> Self {
        match c {
            ColorPref::Black => color::ColorPref::Black,
            ColorPref::White => color::ColorPref::White,
            ColorPref::Any => color::ColorPref::Any,
        }
    }
}
