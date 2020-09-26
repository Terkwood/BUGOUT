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
    use std::rc::Rc;

    struct SGNone;
    struct SGOne(pub SessionGame);
    struct SGTwo(pub SessionGame, pub SessionGame);

    struct PrefsNone;
    struct PrefsOne(pub SessionColorPref);
    struct PrefsTwo(pub SessionColorPref, pub SessionColorPref);

    impl SessionGameRepo for SGNone {
        fn get(&self, _: &SessionId) -> Result<Option<SessionGame>, FetchErr> {
            Ok(None)
        }

        fn put(&self, _: SessionGame) -> Result<(), WriteErr> {
            panic!()
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

        fn put(&self, _: SessionGame) -> Result<(), WriteErr> {
            panic!()
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

        fn put(&self, _: SessionGame) -> Result<(), WriteErr> {
            panic!()
        }
    }

    impl PrefsRepo for PrefsNone {
        fn get(&self, _: &SessionId) -> Result<Option<SessionColorPref>, FetchErr> {
            Ok(None)
        }

        fn put(&self, _: &SessionColorPref) -> Result<(), WriteErr> {
            panic!()
        }
    }

    impl PrefsRepo for PrefsOne {
        fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr> {
            if session_id == &self.0.session_id {
                Ok(Some(self.0.clone()))
            } else {
                Ok(None)
            }
        }

        fn put(&self, _: &SessionColorPref) -> Result<(), WriteErr> {
            panic!()
        }
    }

    impl PrefsRepo for PrefsTwo {
        fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr> {
            if session_id == &self.0.session_id {
                Ok(Some(self.0.clone()))
            } else if session_id == &self.1.session_id {
                Ok(Some(self.1.clone()))
            } else {
                Ok(None)
            }
        }

        fn put(&self, _: &SessionColorPref) -> Result<(), WriteErr> {
            panic!()
        }
    }

    fn new_session_id() -> SessionId {
        SessionId(uuid::Uuid::new_v4())
    }
    fn new_game_id() -> GameId {
        GameId(uuid::Uuid::new_v4())
    }
    fn new_client_id() -> ClientId {
        ClientId(uuid::Uuid::new_v4())
    }

    #[test]
    fn test_by_session_id_complete() {
        let sid = new_session_id();
        let cid = new_client_id();
        let gid = new_game_id();
        let one_pref = SessionColorPref {
            session_id: sid.clone(),
            color_pref: ColorPref::Black,
            client_id: cid.clone(),
        };
        let another_sid = new_session_id();
        let another_cid = new_client_id();
        let another_pref = SessionColorPref {
            session_id: another_sid.clone(),
            color_pref: ColorPref::Black,
            client_id: another_cid.clone(),
        };

        let repos = Repos {
            prefs: Rc::new(PrefsTwo(one_pref.clone(), another_pref.clone())),
            session_game: Rc::new(SGTwo(
                SessionGame {
                    session_id: sid.clone(),
                    game_id: gid.clone(),
                },
                SessionGame {
                    session_id: another_sid.clone(),
                    game_id: gid.clone(),
                },
            )),
        };

        let actual = by_session_id(&sid, &repos).expect("ok");
        assert_eq!(
            actual,
            GameColorPref::Complete {
                game_id: gid,
                prefs: (one_pref, another_pref)
            }
        )
    }

    #[test]
    fn test_by_session_id_partial() {
        let sid = new_session_id();
        let cid = new_client_id();
        let gid = new_game_id();
        let pref = SessionColorPref {
            session_id: sid.clone(),
            color_pref: ColorPref::Black,
            client_id: cid.clone(),
        };
        let repos = Repos {
            prefs: Rc::new(PrefsOne(pref.clone())),
            session_game: Rc::new(SGOne(SessionGame {
                session_id: sid.clone(),
                game_id: gid.clone(),
            })),
        };

        let actual = by_session_id(&sid, &repos).expect("ok");
        assert_eq!(actual, GameColorPref::Partial { game_id: gid, pref })
    }
    #[test]
    fn test_by_session_id_not_ready() {
        let sid = new_session_id();
        let cid = new_client_id();
        let repos = Repos {
            prefs: Rc::new(PrefsOne(SessionColorPref {
                session_id: sid.clone(),
                color_pref: ColorPref::Black,
                client_id: cid.clone(),
            })),
            session_game: Rc::new(SGNone),
        };

        let actual = by_session_id(&sid, &repos).expect("ok");
        assert_eq!(actual, GameColorPref::NotReady)
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
