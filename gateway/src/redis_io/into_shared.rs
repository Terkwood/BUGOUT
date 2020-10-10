use crate::backend_commands::*;
use crate::model::{Coord, Move, Player, ProvideHistoryCommand, Visibility};

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

impl IntoShared<moves::Player> for crate::model::Player {
    fn into_shared(&self) -> moves::Player {
        match self {
            Player::BLACK => moves::Player::BLACK,
            Player::WHITE => moves::Player::WHITE,
        }
    }
}

impl IntoShared<Option<moves::Coord>> for Option<Coord> {
    fn into_shared(&self) -> Option<moves::Coord> {
        self.map(|Coord { x, y }| moves::Coord { x, y })
    }
}

impl IntoShared<sync::Move> for Move {
    fn into_shared(&self) -> sync::Move {
        sync::Move {
            player: self.player.into_shared(),
            turn: self.turn as u32,
            coord: self.coord.into_shared(),
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
            game_id: core::GameId(self.game_id),
            client_id: core::ClientId(self.client_id),
            session_id: core::SessionId(self.session_id),
        }
    }
}

impl IntoShared<lobby::api::FindPublicGame> for FindPublicGameBackendCommand {
    fn into_shared(&self) -> lobby::api::FindPublicGame {
        lobby::api::FindPublicGame {
            client_id: core::ClientId(self.client_id),
            session_id: core::SessionId(self.session_id),
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

impl IntoShared<color::api::ChooseColorPref> for ChooseColorPrefBackendCommand {
    fn into_shared(&self) -> color::api::ChooseColorPref {
        todo!()
    }
}
