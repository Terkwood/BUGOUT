use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Coord {
    x: u16,
    y: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Player {
    BLACK,
    WHITE,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Commands {
    MakeMove {
        #[serde(rename = "gameId")]
        game_id: Uuid,
        #[serde(rename = "reqId")]
        req_id: Uuid,
        player: Player,
        coord: Option<Coord>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Events {
    MoveMade {
        #[serde(rename = "gameId")]
        game_id: Uuid,
        #[serde(rename = "replyTo")]
        reply_to: Uuid,
        #[serde(rename = "eventId")]
        event_id: Uuid,
        player: Player,
        coord: Option<Coord>,
        captured: Vec<Coord>,
    },
    MoveRejected {
        game_id: Uuid,
        reply_to: Uuid,
        event_id: Uuid,
        player: Player,
        coord: Coord,
    },
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    #[test]
    fn serialize_move_command() {
        let game_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();

        assert_eq!(
            serde_json::to_string(&super::Commands::MakeMove {
                game_id,
                req_id,
                player: super::Player::BLACK,
                coord: Some(super::Coord { x: 0, y: 0 })
            })
            .unwrap(),
            format!("{{\"type\":\"MakeMove\",\"gameId\":\"{:?}\",\"reqId\":\"{:?}\",\"player\":\"BLACK\",\"coord\":{{\"x\":0,\"y\":0}}}}", game_id, request_id)
        )
    }
}
