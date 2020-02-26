const DEFAULT_MAKE_MOVE_CMD: &str = "bugout-make-move-cmd";
const DEFAULT_GAME_STATES_CHANGELOG: &str = "bugout-game-states";
const DEFAULT_MOVE_ACCEPTED_EV: &str = "bugout-move-accepted-ev";

#[derive(Clone)]
pub struct StreamTopics {
    pub make_move_cmd: String,
    pub game_states_changelog: String,
    pub move_accepted_ev: String,
}
impl Default for StreamTopics {
    fn default() -> Self {
        StreamTopics {
            make_move_cmd: DEFAULT_MAKE_MOVE_CMD.to_string(),
            game_states_changelog: DEFAULT_GAME_STATES_CHANGELOG.to_string(),
            move_accepted_ev: DEFAULT_MOVE_ACCEPTED_EV.to_string(),
        }
    }
}
