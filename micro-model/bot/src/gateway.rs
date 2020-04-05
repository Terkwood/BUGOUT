use micro_model_moves::{GameId, Player};
use serde_derive::{Deserialize, Serialize};

/// This command is sent from gateway, and
/// requests that robocall coordinate with
/// tinybrain to generate moves for a given
/// game ID and player.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttachBot {
    pub game_id: GameId,
    pub player: Player,
    pub board_size: Option<u8>,
}

/// This reply is sent once a bot is listening
/// as a certain player in a certain game.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BotAttached {
    pub game_id: GameId,
    pub player: Player,
}

impl AttachBot {
    pub fn from(bytes: &[u8]) -> Result<Self, std::boxed::Box<bincode::ErrorKind>> {
        bincode::deserialize(bytes)
    }
    pub fn serialize(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        Ok(bincode::serialize(&self)?)
    }
}
impl BotAttached {
    pub fn from(bytes: &[u8]) -> Result<Self, std::boxed::Box<bincode::ErrorKind>> {
        bincode::deserialize(bytes)
    }
    pub fn serialize(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        Ok(bincode::serialize(&self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Player;
    use uuid::Uuid;

    #[test]
    fn test_attach_bot_json() {
        let expected = AttachBot {
            game_id: GameId(Uuid::nil()),
            player: Player::BLACK,
            board_size: Some(9),
        };
        let json = serde_json::to_string(&expected).expect("to_string");
        let actual: AttachBot = serde_json::from_str(&json).expect("from_str");
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_bot_attached_json() {
        let expected = BotAttached {
            game_id: GameId(Uuid::new_v4()),
            player: Player::WHITE,
        };
        let json = serde_json::to_string(&expected).expect("to_string");
        let actual: BotAttached = serde_json::from_str(&json).expect("from_str");
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_camel_case_attach_bot() {
        let input = AttachBot {
            game_id: GameId(Uuid::nil()),
            player: Player::BLACK,
            board_size: Some(19),
        };
        let json = serde_json::to_string(&input).expect("to_string");
        assert!(json.contains("gameId"));
        assert!(json.contains("boardSize"))
    }
}
