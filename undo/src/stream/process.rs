use super::undo::consume_undo;
use super::*;
use crate::repo::Botness;
use crate::Components;
use log::error;

fn consume_log(game_state: &GameState, reg: &Components) {
    if let Err(e) = reg.game_state_repo.put(&game_state) {
        error!("could not track game state: {:?}", e)
    }
}

fn consume_bot_attached(ba: &BotAttached, reg: &Components) {
    if let Err(e) = reg.botness_repo.put(&ba.game_id, ba.player, Botness::IsBot) {
        error!("could not track bot attached: {:?}", e)
    }
}
