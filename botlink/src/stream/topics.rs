const DEFAULT_ATTACH_BOT_EV: &str = "bugout-attach-bot-ev";
const DEFAULT_GAME_STATES_CHANGELOG: &str = "bugout-game-states";
const DEFAULT_MAKE_MOVE_CMD: &str = "bugout-make-move-cmd";

#[derive(Clone, Debug)]
pub struct Topics {
    pub attach_bot_ev: String,
    pub game_states_changelog: String,
    pub make_move_cmd: String,
}
impl Default for Topics {
    fn default() -> Self {
        Topics {
            game_states_changelog: DEFAULT_GAME_STATES_CHANGELOG.to_string(),
            make_move_cmd: DEFAULT_MAKE_MOVE_CMD.to_string(),
            attach_bot_ev: DEFAULT_ATTACH_BOT_EV.to_string(),
        }
    }
}
