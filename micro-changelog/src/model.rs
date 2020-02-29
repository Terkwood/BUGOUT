use micro_model_moves::GameId;
#[derive(Debug, Clone)]
pub struct GameReadyEvent {
    game_id: GameId,
}
