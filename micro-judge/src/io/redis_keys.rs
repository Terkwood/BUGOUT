use crate::model::GameId;
const DEFAULT_NAMESPACE: &str = "BUGOUT";
#[derive(Clone)]
pub struct Namespace(pub String);
impl Default for Namespace {
    fn default() -> Self {
        Namespace(DEFAULT_NAMESPACE.to_string())
    }
}
pub fn entry_ids_hash_key(namespace: &Namespace) -> String {
    format!("/{}/micro_judge/entry_ids", namespace.0)
}
pub fn game_states_key(namespace: &Namespace, game_id: &GameId) -> String {
    format!("/{}/micro_judge/game_states/{}", namespace.0, game_id.0)
}
