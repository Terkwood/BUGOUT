use core_model::GameId;
const DEFAULT_NAMESPACE: &str = "BUGOUT";
#[derive(Clone, Debug)]
pub struct RedisKeyNamespace(pub String);
impl Default for RedisKeyNamespace {
    fn default() -> Self {
        RedisKeyNamespace(DEFAULT_NAMESPACE.to_string())
    }
}
pub fn entry_ids_hash_key(namespace: &RedisKeyNamespace) -> String {
    format!("/{}/micro_judge/entry_ids", namespace.0)
}
pub fn game_states_key(namespace: &RedisKeyNamespace, game_id: &GameId) -> String {
    format!("/{}/micro_judge/game_states/{}", namespace.0, game_id.0)
}
