use crate::backend_commands::BackendCommands as BC;
use crate::backend_commands::{
    CreateGameBackendCommand, FindPublicGameBackendCommand, JoinPrivateGameBackendCommand,
};
use crate::model::{Coord, MakeMoveCommand, ProvideHistoryCommand};
use crate::redis_io::RedisPool;
use crate::topics;
use micro_model_bot::gateway::AttachBot;

use crossbeam_channel::{select, Receiver};
use log::error;
use r2d2_redis::redis;
use std::sync::Arc;

pub trait IntoShared<T> {
    fn into_shared(&self) -> T;
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
