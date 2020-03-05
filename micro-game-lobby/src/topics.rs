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

const DEFAULT_FIND_PUBLIC_GAME: &str = "bugout-find-public-game-cmd";
const DEFAULT_CREATE_GAME: &str = "bugout-create-game-cmd";
const DEFAULT_WAIT_FOR_OPPONENT: &str = "bugout-wait-for-opponent-ev";
const DEFAULT_GAME_READY: &str = "bugout-game-ready-ev";
const DEFAULT_PRIVATE_GAME_REJECTED: &str = "bugout-private-game-rejected-ev";
const DEFAULT_JOIN_PRIVATE_GAME: &str = "bugout-join-private-game-cmd";
const DEFAULT_SESSION_DISCONNECTED: &str = "bugout-session-disconnected-ev";

impl Default for StreamTopics {
    fn default() -> Self {
        StreamTopics {
            find_public_game: DEFAULT_FIND_PUBLIC_GAME.to_string(),
            create_game: DEFAULT_CREATE_GAME.to_string(),
            join_private_game: DEFAULT_JOIN_PRIVATE_GAME.to_string(),
            wait_for_opponent: DEFAULT_WAIT_FOR_OPPONENT.to_string(),
            game_ready: DEFAULT_GAME_READY.to_string(),
            private_game_rejected: DEFAULT_PRIVATE_GAME_REJECTED.to_string(),
            session_disconnected: DEFAULT_SESSION_DISCONNECTED.to_string(),
        }
    }
}
