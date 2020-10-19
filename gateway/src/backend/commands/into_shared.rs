use crate::backend::commands::*;
use crate::model::ProvideHistoryCommand;
use uuid::Uuid;

use color_model as color;
use core_model as core;
use lobby_model as lobby;
use sync_model as sync;

pub trait IntoShared<T> {
    fn into_shared(&self) -> T;
}

impl IntoShared<color::api::ChooseColorPref> for ChooseColorPrefBackendCommand {
    fn into_shared(&self) -> color::api::ChooseColorPref {
        color::api::ChooseColorPref {
            session_id: self.session_id.into_shared(),
            client_id: self.client_id.into_shared(),
            color_pref: self.color_pref.into(),
        }
    }
}

impl IntoShared<sync::api::ReqSync> for ReqSyncBackendCommand {
    fn into_shared(&self) -> sync::api::ReqSync {
        sync::api::ReqSync {
            session_id: self.session_id.into_shared(),
            req_id: self.req_id.into_shared(),
            game_id: self.game_id.into_shared(),
            player_up: self.player_up.into(),
            last_move: self.last_move.map(|m| m.into()),
            turn: self.turn,
        }
    }
}

impl IntoShared<lobby::api::SessionDisconnected> for SessionDisconnected {
    fn into_shared(&self) -> lobby::api::SessionDisconnected {
        lobby::api::SessionDisconnected {
            session_id: self.session_id.into_shared(),
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

impl IntoShared<lobby::api::CreateGame> for CreateGameBackendCommand {
    fn into_shared(&self) -> lobby::api::CreateGame {
        lobby::api::CreateGame {
            client_id: self.client_id.into_shared(),
            game_id: None,
            visibility: self.visibility.into(),
            session_id: self.session_id.into_shared(),
            board_size: self.board_size,
        }
    }
}

impl IntoShared<sync::api::ProvideHistory> for ProvideHistoryCommand {
    fn into_shared(&self) -> sync::api::ProvideHistory {
        sync::api::ProvideHistory {
            game_id: self.game_id.into_shared(),
            req_id: self.req_id.into_shared(),
        }
    }
}

impl IntoShared<core::GameId> for Uuid {
    fn into_shared(&self) -> core::GameId {
        core::GameId(self.clone())
    }
}

impl IntoShared<core::ReqId> for Uuid {
    fn into_shared(&self) -> core::ReqId {
        core::ReqId(self.clone())
    }
}

impl IntoShared<core::EventId> for Uuid {
    fn into_shared(&self) -> core::EventId {
        core::EventId(self.clone())
    }
}

impl IntoShared<core::SessionId> for Uuid {
    fn into_shared(&self) -> core::SessionId {
        core::SessionId(self.clone())
    }
}

impl IntoShared<core::ClientId> for Uuid {
    fn into_shared(&self) -> core::ClientId {
        core::ClientId(self.clone())
    }
}
