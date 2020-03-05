#[derive(Debug, Clone)]
pub struct StreamTopics {
    pub find_public_game: String,
    pub create_game: String,
    pub join_private_game: String,
    pub wait_for_opponent: String,
    pub game_ready: String,
    pub private_game_rejected: String,
    pub session_disconnected: String,
}
