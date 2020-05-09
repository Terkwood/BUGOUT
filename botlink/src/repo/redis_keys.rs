pub const ENTRY_IDS: &str = "/BUGOUT/botlink/entry_ids";
pub const ATTACHED_BOTS: &str = "/BUGOUT/botlink/attached_bots";
const NAMESPACE: &str = "BUGOUT";

pub fn board_size(game_id: &uuid::Uuid) -> String {
    format!("/{}/botlink/board_size/{}", NAMESPACE, game_id.to_string())
}
