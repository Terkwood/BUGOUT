#[derive(Clone, Debug)]
pub enum StreamInput {
    AB(micro_model_bot::gateway::AttachBot),
    GS(move_model::GameState),
}
