use micro_model_moves::GameId;

const DEFAULT_NAMESPACE: &str = "BUGOUT";
#[derive(Clone, Debug)]
pub struct RedisKeyNamespace(pub String);
impl Default for RedisKeyNamespace {
    fn default() -> Self {
        RedisKeyNamespace(DEFAULT_NAMESPACE.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct GameStatesHashKey(pub RedisKeyNamespace);

impl GameStatesHashKey {
    pub fn value(&self, game_id: &GameId) -> String {
        format!("/{}/micro_changelog/game_states/{}", (self.0).0, game_id.0)
    }
}
