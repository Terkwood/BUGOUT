use micro_model_moves::GameId;
#[derive(Debug, Clone)]
pub struct GameReadyEvent {
    pub game_id: GameId,
}
