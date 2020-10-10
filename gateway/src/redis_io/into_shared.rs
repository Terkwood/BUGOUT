use crate::backend_commands::*;
use crate::model::ProvideHistoryCommand;

use color_model as color;
use core_model as core;
use lobby_model as lobby;
use sync_model as sync;

pub trait IntoShared<T> {
    fn into_shared(&self) -> T;
}

impl IntoShared<sync::api::ProvideHistory> for ProvideHistoryCommand {
    fn into_shared(&self) -> sync::api::ProvideHistory {
        todo!()
    }
}

impl IntoShared<sync::api::ReqSync> for ReqSyncBackendCommand {
    fn into_shared(&self) -> sync::api::ReqSync {
        todo!()
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

impl IntoShared<lobby::api::CreateGame> for CreateGameBackendCommand {
    fn into_shared(&self) -> lobby::api::CreateGame {
        todo!()
    }
}

impl IntoShared<color::api::ChooseColorPref> for ChooseColorPrefBackendCommand {
    fn into_shared(&self) -> color::api::ChooseColorPref {
        todo!()
    }
}
