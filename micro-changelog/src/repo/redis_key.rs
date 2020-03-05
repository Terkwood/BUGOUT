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
pub struct KeyProvider(pub RedisKeyNamespace);
impl Default for KeyProvider {
    fn default() -> Self {
        KeyProvider(RedisKeyNamespace::default())
    }
}
impl KeyProvider {
    pub fn game_states(&self, game_id: &GameId) -> String {
        format!("/{}/micro_changelog/game_states/{}", (self.0).0, game_id.0)
    }
    pub fn entry_ids(&self) -> String {
        format!("/{}/micro_changelog/entry_ids", (self.0).0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tesh_key_prov_default() {
        let g = KeyProvider::default();
        assert_eq!(
            g.game_states(&GameId(uuid::Uuid::nil())),
            "/BUGOUT/micro_changelog/game_states/00000000-0000-0000-0000-000000000000"
        );
    }
}
