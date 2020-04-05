const DEFAULT_GAME_STATES_CHANGELOG: &str = "bugout-game-states";
const DEFAULT_MOVE_ACCEPTED_EV: &str = "bugout-move-accepted-ev";
const DEFAULT_MOVE_MADE_EV: &str = "bugout-move-made-ev";

#[derive(Clone, Debug)]
pub struct StreamTopics {
    pub game_states_changelog: String,
    pub move_accepted_ev: String,
    pub move_made_ev: String,
}
impl Default for StreamTopics {
    fn default() -> Self {
        StreamTopics {
            game_states_changelog: DEFAULT_GAME_STATES_CHANGELOG.to_string(),
            move_accepted_ev: DEFAULT_MOVE_ACCEPTED_EV.to_string(),
            move_made_ev: DEFAULT_MOVE_MADE_EV.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stream_topics_default() {
        assert_eq!(
            &StreamTopics::default().game_states_changelog,
            DEFAULT_GAME_STATES_CHANGELOG
        );
        assert_eq!(
            &StreamTopics::default().move_accepted_ev,
            DEFAULT_MOVE_ACCEPTED_EV
        );
        assert_eq!(&StreamTopics::default().move_made_ev, DEFAULT_MOVE_MADE_EV);
    }
}
