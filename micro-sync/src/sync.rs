use crate::api::ReqSync;
use crate::model::Player;
use crate::player::other_player;

/// Note that the check for last_move.is_some() makes
/// sense.  If the player passed, we'll still see a `Move`,
/// but its `coord` field will be empty.
pub fn is_client_ahead_by_one_turn(
    req_sync: &ReqSync,
    system_turn: u32,
    system_player_up: Player,
) -> bool {
    req_sync.turn == system_turn + 1
        && req_sync.player_up == other_player(system_player_up)
        && req_sync.last_move.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    #[test]
    fn expected_client_ahead_by_one() {
        let game_id = GameId::random();
        let session_id = SessionId::random();
        let req_id = ReqId::random();

        let system_turn = 3;
        let system_player_up = Player::BLACK;
        let req = ReqSync {
            player_up: Player::WHITE,
            turn: system_turn - 2,
            last_move: Some(Move {
                player: Player::BLACK,
                coord: None,
                turn: 1,
            }),
            game_id: game_id.clone(),
            session_id: session_id.clone(),
            req_id: req_id.clone(),
        };
        assert!(is_client_ahead_by_one_turn(
            &req,
            system_turn,
            system_player_up
        ))
    }
}
