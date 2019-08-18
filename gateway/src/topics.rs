pub const BROKERS: &str = "kafka:9092";
pub const APP_NAME: &str = "gateway";
pub const GAME_STATES_TOPIC: &str = "bugout-game-states";
pub const MAKE_MOVE_TOPIC: &str = "bugout-make-move-cmd";
pub const MOVE_MADE_TOPIC: &str = "bugout-move-made-ev";
pub const PROVIDE_HISTORY_TOPIC: &str = "bugout-provide-history-cmd";
pub const HISTORY_PROVIDED_TOPIC: &str = "bugout-history-provided-ev";
pub const JOIN_PRIVATE_GAME_TOPIC: &str = "bugout-join-private-game-cmd";
pub const PRIVATE_GAME_REJECTED_TOPIC: &str = "bugout-private-game-rejected-ev";
pub const GAME_READY_TOPIC: &str = "bugout-game-ready-ev";
pub const CONSUME_TOPICS: &[&str] = &[
    MOVE_MADE_TOPIC,
    HISTORY_PROVIDED_TOPIC,
    PRIVATE_GAME_REJECTED_TOPIC,
    GAME_READY_TOPIC,
];
