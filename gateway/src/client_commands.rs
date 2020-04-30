use serde_derive::{Deserialize, Serialize};

use crate::compact_ids::CompactId;
use crate::model::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReconnectCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JoinPrivateGameClientCommand {
    #[serde(rename = "gameId")]
    pub game_id: CompactId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreatePrivateGameClientCommand {
    #[serde(rename = "boardSize")]
    pub board_size: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChooseColorPrefClientCommand {
    #[serde(rename = "colorPref")]
    pub color_pref: ColorPref,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AttachBotClientCommand {
    pub player: Player,
    #[serde(rename = "boardSize")]
    pub board_size: Option<u8>,
}

/// This event requests synchronization with the backend
/// view of the game.  This event includes a request ID
/// which should be remembered by the browser client.
/// Any future sync replies should include a reply_to
/// which ties to the most recent ReqSync sent by a
/// client.  This helps clients avoid reacting to
/// stale updates.
///
/// This event includes the last known move to happen
/// from the client's point of view, which can help
/// prevent the situation where a client sends a move
/// via websocket, which is never processed by the backend.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReqSyncClientCommand {
    pub req_id: ReqId,
    pub player_up: Player,
    pub turn: u32,
    pub last_move: Option<Move>,
}

/// Events originating from the browser and
/// being sent to gateway
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ClientCommands {
    MakeMove(MakeMoveCommand),
    Beep,
    Reconnect(ReconnectCommand),
    ProvideHistory(ProvideHistoryCommand),
    JoinPrivateGame(JoinPrivateGameClientCommand),
    FindPublicGame,
    CreatePrivateGame(CreatePrivateGameClientCommand),
    ChooseColorPref(ChooseColorPrefClientCommand),
    ProvideIdleStatus,
    Identify(Identity),
    QuitGame,
    AttachBot(AttachBotClientCommand),
    ReqSync(ReqSyncClientCommand),
}

#[cfg(test)]
mod tests {
    use crate::client_commands::*;
    use crate::compact_ids::CompactId;
    use uuid::Uuid;

    #[test]
    fn serialize_move_command() {
        let game_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();

        assert_eq!(
            serde_json::to_string(&super::ClientCommands::MakeMove (super::MakeMoveCommand{
                game_id,
                req_id,
                player: super::Player::BLACK,
                coord: Some(super::Coord { x: 0, y: 0 })
            }))
            .unwrap(),
            format!("{{\"type\":\"MakeMove\",\"gameId\":\"{:?}\",\"reqId\":\"{:?}\",\"player\":\"BLACK\",\"coord\":{{\"x\":0,\"y\":0}}}}", game_id, req_id)
        )
    }

    #[test]
    fn deserialize_join_priv_game_client_command() {
        let compact_game_id = CompactId::encode(Uuid::new_v4());

        let json = &format!(
            "{{\"type\":\"JoinPrivateGame\",\"gameId\":\"{}\"}}",
            compact_game_id.0
        );

        let d: ClientCommands = serde_json::from_str(json).unwrap();

        assert_eq!(
            d,
            ClientCommands::JoinPrivateGame(JoinPrivateGameClientCommand {
                game_id: compact_game_id
            })
        )
    }

    #[test]
    fn deserialize_find_public_game_client_command() {
        let json = "{\"type\":\"FindPublicGame\"}";

        let d: ClientCommands = serde_json::from_str(json).unwrap();

        assert_eq!(d, ClientCommands::FindPublicGame)
    }

    #[test]
    fn deserialize_create_private_game_client_command() {
        let json = "{\"type\":\"CreatePrivateGame\"}";

        let d: ClientCommands = serde_json::from_str(json).unwrap();

        assert_eq!(
            d,
            ClientCommands::CreatePrivateGame(CreatePrivateGameClientCommand { board_size: None })
        )
    }

    #[test]
    fn deserialize_create_private_game_board_size_9() {
        let json = "{\"type\":\"CreatePrivateGame\", \"boardSize\":9}";

        let d: ClientCommands = serde_json::from_str(json).unwrap();

        assert_eq!(
            d,
            ClientCommands::CreatePrivateGame(CreatePrivateGameClientCommand {
                board_size: Some(9)
            })
        )
    }

    #[test]
    fn deserialize_beep_client_command() {
        let json = "{\"type\":\"Beep\"}";

        let d: ClientCommands = serde_json::from_str(json).unwrap();

        assert_eq!(d, ClientCommands::Beep)
    }
}
