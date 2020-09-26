use crate::api::GameReady;
use crate::model::*;
use crate::repo::*;
use crate::Components;
use std::rc::Rc;

/// Call this when you receive a ChooseColorPref event

pub fn by_session_id(session_id: &SessionId, repos: &Repos) -> Result<GameColorPref, FetchErr> {
    todo!()
}

/// Call this when you receive a GameReady event.
/// It will provide an aggregated view of choices for that game,
/// based on all available data from both session_game repo
/// and prefs repo.
pub fn by_game_ready(game_id: &GameReady, repos: &Repos) -> Result<GameColorPref, FetchErr> {
    todo!()
}
