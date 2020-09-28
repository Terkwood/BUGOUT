use crate::api::ReqSync;
use crate::model::{Move, Player};
pub fn is_client_ahead_by_one_turn(
    req_sync: &ReqSync,
    system_history: Vec<Move>,
    system_turn: u32,
    system_player_up: Player,
) -> bool {
    todo!()
}
