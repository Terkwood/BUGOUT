use serde_derive::{Deserialize, Serialize};

use crate::model::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinPrivateGameKafkaCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FindPublicGameKafkaCommand {
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
}

/// Gateway may manually create private games,
/// but it will never create a public game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateGameKafkaCommand {
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    pub visibility: Visibility,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KafkaCommands {
    MakeMove(MakeMoveCommand),
    ProvideHistory(ProvideHistoryCommand),
    JoinPrivateGame(JoinPrivateGameKafkaCommand),
    FindPublicGame(FindPublicGameKafkaCommand),
    CreateGame(CreateGameKafkaCommand),
}
