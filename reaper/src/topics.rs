pub const MAKE_MOVE_TOPIC: &str = "bugout-make-move-cmd";
pub const PROVIDE_HISTORY_TOPIC: &str = "bugout-provide-history-cmd";
pub const JOIN_PRIVATE_GAME_TOPIC: &str = "bugout-join-private-game-cmd";
pub const FIND_PUBLIC_GAME_TOPIC: &str = "bugout-find-public-game-cmd";
pub const CREATE_GAME_TOPIC: &str = "bugout-create-game-cmd";
pub const CHOOSE_COLOR_PREF_TOPIC: &str = "bugout-choose-color-pref-cmd";

pub const MOVE_MADE_TOPIC: &str = "bugout-move-made-ev";
pub const HISTORY_PROVIDED_TOPIC: &str = "bugout-history-provided-ev";
pub const PRIVATE_GAME_REJECTED_TOPIC: &str = "bugout-private-game-rejected-ev";
pub const GAME_READY_TOPIC: &str = "bugout-game-ready-ev";
pub const WAIT_FOR_OPPONENT_TOPIC: &str = "bugout-wait-for-opponent-ev";
pub const COLORS_CHOSEN_TOPIC: &str = "bugout-colors-chosen-ev";
pub const CLIENT_HEARTBEAT_TOPIC: &str = "bugout-client-heartbeat-ev";

pub const SHUTDOWN_TOPIC: &str = "bugout-shutdown-ev";

pub const CONSUME_TOPICS: &[&str] = &[
    MAKE_MOVE_TOPIC,
    PROVIDE_HISTORY_TOPIC,
    JOIN_PRIVATE_GAME_TOPIC,
    FIND_PUBLIC_GAME_TOPIC,
    CREATE_GAME_TOPIC,
    CHOOSE_COLOR_PREF_TOPIC,
    MOVE_MADE_TOPIC,
    HISTORY_PROVIDED_TOPIC,
    PRIVATE_GAME_REJECTED_TOPIC,
    GAME_READY_TOPIC,
    WAIT_FOR_OPPONENT_TOPIC,
    COLORS_CHOSEN_TOPIC,
    CLIENT_HEARTBEAT_TOPIC,
];
