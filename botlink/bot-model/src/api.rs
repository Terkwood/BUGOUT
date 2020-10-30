use super::{AlphaNumCoord, Bot};
use core_model::GameId;
use move_model::{GameState, Player};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMove {
    pub game_id: GameId,
    pub game_state: GameState,
    pub max_visits: Option<u16>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveComputed {
    pub game_id: GameId,
    pub player: Player,
    pub alphanum_coord: Option<AlphaNumCoord>,
}

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
    pub bot: Bot,
}

/// This reply is sent once a bot is listening
/// as a certain player in a certain game.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BotAttached {
    pub game_id: GameId,
    pub player: Player,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_attach_bot_json() {
        let expected = AttachBot {
            game_id: GameId(Uuid::nil()),
            player: Player::BLACK,
            board_size: Some(9),
            bot: Bot::KataGoInstant,
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
            bot: Bot::KataGoFullStrength,
        };
        let json = serde_json::to_string(&input).expect("to_string");
        assert!(json.contains("gameId"));
        assert!(json.contains("boardSize"));
        assert!(json.contains("bot"));
    }
}
