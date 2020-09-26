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
    todo!("check session prefs")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_by_session_id() {
        todo!()
    }
    #[test]
    fn test_by_game_ready() {
        todo!()
    }
}
