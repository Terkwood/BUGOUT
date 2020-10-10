use crate::backend_commands::*;
use crate::model::ProvideHistoryCommand;

pub trait IntoShared<T> {
    fn into_shared(&self) -> T;
}

impl IntoShared<sync_model::api::ProvideHistory> for ProvideHistoryCommand {
    fn into_shared(&self) -> sync_model::api::ProvideHistory {
        todo!()
    }
}

impl IntoShared<sync_model::api::ReqSync> for ReqSyncBackendCommand {
    fn into_shared(&self) -> sync_model::api::ReqSync {
        todo!()
    }
}

impl IntoShared<lobby_model::api::JoinPrivateGame> for JoinPrivateGameBackendCommand {
    fn into_shared(&self) -> lobby_model::api::JoinPrivateGame {
        use core_model as cm;

        lobby_model::api::JoinPrivateGame {
            game_id: cm::GameId(self.game_id),
            client_id: cm::ClientId(self.client_id),
            session_id: cm::SessionId(self.session_id),
        }
    }
}

impl IntoShared<lobby_model::api::FindPublicGame> for FindPublicGameBackendCommand {
    fn into_shared(&self) -> lobby_model::api::FindPublicGame {
        todo!()
    }
}

impl IntoShared<lobby_model::api::CreateGame> for CreateGameBackendCommand {
    fn into_shared(&self) -> lobby_model::api::CreateGame {
        todo!()
    }
}
