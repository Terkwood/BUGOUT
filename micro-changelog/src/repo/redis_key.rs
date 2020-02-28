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
pub struct GameStatesHashKeyProvider(pub RedisKeyNamespace);
impl Default for GameStatesHashKeyProvider {
    fn default() -> Self {
        GameStatesHashKeyProvider(RedisKeyNamespace::default())
    }
}
impl GameStatesHashKeyProvider {
    pub fn value(&self, game_id: &GameId) -> String {
        format!("/{}/micro_changelog/game_states/{}", (self.0).0, game_id.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tesh_hash_key_prov_default() {
        let g = GameStatesHashKeyProvider::default();
        assert_eq!(
            g.value(&GameId(uuid::Uuid::nil())),
            "/BUGOUT/micro_changelog/game_states/00000000-0000-0000-0000-000000000000"
        );
    }
}
