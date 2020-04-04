use serde_derive::{Deserialize, Serialize};

use crate::model::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinPrivateGameBackendCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FindPublicGameBackendCommand {
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChooseColorPrefBackendCommand {
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    #[serde(rename = "colorPref")]
    pub color_pref: ColorPref,
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
}

/// Gateway may manually create private games,
/// but it will never create a public game.
/// We omit specifying the game ID here, and
/// let game lobby choose it for us.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateGameBackendCommand {
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    pub visibility: Visibility,
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
    #[serde(rename = "boardSize")]
    pub board_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum HeartbeatType {
    WebSocketPong,
    UserInterfaceBeep,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientHeartbeat {
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    #[serde(rename = "heartbeatType")]
    pub heartbeat_type: HeartbeatType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionDisconnected {
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuitGameCommand {
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    #[serde(rename = "gameId")]
    pub game_id: GameId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BackendCommands {
    MakeMove(MakeMoveCommand),
    ProvideHistory(ProvideHistoryCommand),
    JoinPrivateGame(JoinPrivateGameBackendCommand),
    FindPublicGame(FindPublicGameBackendCommand),
    CreateGame(CreateGameBackendCommand),
    ChooseColorPref(ChooseColorPrefBackendCommand),
    ClientHeartbeat(ClientHeartbeat),
    SessionDisconnected(SessionDisconnected),
    QuitGame(QuitGameCommand),
}

#[derive(Clone, Debug)]
pub enum SessionCommands {
    StartBotSession {
        session_id: SessionId,
        bot_player: crate::model::Player,
        board_size: Option<u8>,
    },
    Backend {
        session_id: SessionId,
        command: BackendCommands,
    },
}
