use core_model::GameId;
#[derive(Debug, Clone)]
pub struct GameReadyEvent {
    pub game_id: GameId,
}
