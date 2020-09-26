use crate::api::GameReady;
use crate::components::Repos;
use crate::model::*;
use crate::repo::*;

/// Call this when you receive a ChooseColorPref event
/// It will provide an aggregated view of choices for that game,
/// based on all available data from both session_game repo
/// and prefs repo.
pub fn by_session_id(session_id: &SessionId, repos: &Repos) -> Result<GameColorPref, FetchErr> {
    todo!("when there is no link from session ID to game, it's empty");
    todo!("when there is a link, but only one person's prefs registered, partial");
    todo!("otherwise complete")
}

/// Call this when you receive a GameReady event.
/// It will provide an aggregated view of choices for that game,
/// based on all available data from both session_game repo
/// and prefs repo.
pub fn by_game_ready(game_ready: &GameReady, repos: &Repos) -> Result<GameColorPref, FetchErr> {
    todo!("check session prefs for each mentioned session id");
    todo!("you need both sessions to have prefs in order to return complete");
}

#[cfg(test)]
mod tests {
    use super::*;
    struct SGNone;
    struct SGOne(pub SessionGame);
    struct SGTwo(pub SessionGame, pub SessionGame);

    struct PrefsOne(pub SessionColorPref);
    struct PrefsTwo(pub SessionColorPref);

    impl SessionGameRepo for SGNone {
        fn get(&self, session_id: &SessionId) -> Result<Option<SessionGame>, FetchErr> {
            Ok(None)
        }

        fn put(&self, session_game: SessionGame) -> Result<(), WriteErr> {
            todo!()
        }
    }

    impl SessionGameRepo for SGOne {
        fn get(&self, session_id: &SessionId) -> Result<Option<SessionGame>, FetchErr> {
            if session_id == &self.0.session_id {
                Ok(Some(self.0.clone()))
            } else {
                Ok(None)
            }
        }

        fn put(&self, session_game: SessionGame) -> Result<(), WriteErr> {
            todo!()
        }
    }

    impl SessionGameRepo for SGTwo {
        fn get(&self, session_id: &SessionId) -> Result<Option<SessionGame>, FetchErr> {
            if session_id == &self.0.session_id {
                Ok(Some(self.0.clone()))
            } else if session_id == &self.1.session_id {
                Ok(Some(self.1.clone()))
            } else {
                Ok(None)
            }
        }

        fn put(&self, session_game: SessionGame) -> Result<(), WriteErr> {
            todo!()
        }
    }

    #[test]
    fn test_by_session_id_complete() {
        todo!()
    }

    #[test]
    fn test_by_session_id_partial() {
        todo!()
    }
    #[test]
    fn test_by_session_id_not_ready() {
        todo!()
    }

    #[test]
    fn test_by_game_ready_complete() {
        todo!()
    }
    #[test]
    fn test_by_game_ready_partial() {
        todo!()
    }
    #[test]
    fn test_by_game_ready_not_ready() {
        todo!()
    }
}
