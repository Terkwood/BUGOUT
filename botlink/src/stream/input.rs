#[derive(Clone, Debug)]
pub enum StreamInput {
    AB(bot_model::api::AttachBot),
    GS(move_model::GameState),
}
