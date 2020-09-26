use crate::api::GameReady;
use crate::components::Repos;
use crate::model::*;
use crate::repo::*;

/// Call this when you receive a ChooseColorPref event
/// It will provide an aggregated view of choices for that game,
/// based on all available data from both game_ready repo
/// and prefs repo.
pub fn by_session_id(session_id: &SessionId, repos: &Repos) -> Result<GameColorPref, FetchErr> {
    repos.game_ready.get(session_id).and_then(|sg| match sg {
        None => Ok(GameColorPref::NotReady),
        Some(game_ready) => {
            let first_pref = repos.prefs.get(&game_ready.sessions.0);
            let second_pref = repos.prefs.get(&game_ready.sessions.1);
            match (first_pref, second_pref) {
                (Ok(Some(first)), Ok(Some(second))) => Ok(GameColorPref::Complete {
                    game_id: game_ready.game_id.clone(),
                    prefs: (first, second),
                }),
                (Ok(Some(partial)), Ok(None)) => Ok(GameColorPref::Partial {
                    game_id: game_ready.game_id.clone(),
                    pref: partial,
                }),
                (Ok(None), Ok(Some(partial))) => Ok(GameColorPref::Partial {
                    game_id: game_ready.game_id.clone(),
                    pref: partial,
                }),
                (Ok(None), Ok(None)) => Ok(GameColorPref::NotReady),
                _ => Err(FetchErr),
            }
        }
    })
}

/// Call this when you receive a GameReady event.
/// It will provide an aggregated view of choices for that game,
/// based on all available data from both game_ready repo
/// and prefs repo.
pub fn by_game_ready(game_ready: &GameReady, repos: &Repos) -> Result<GameColorPref, FetchErr> {
    let first_pref = repos.prefs.get(&game_ready.sessions.0)?;
    let second_pref = repos.prefs.get(&game_ready.sessions.1)?;

    Ok(match (first_pref, second_pref) {
        (Some(first), Some(second)) => GameColorPref::Complete {
            game_id: game_ready.game_id.clone(),
            prefs: (first, second),
        },
        (Some(partial), None) => GameColorPref::Partial {
            game_id: game_ready.game_id.clone(),
            pref: partial,
        },
        (None, Some(partial)) => GameColorPref::Partial {
            game_id: game_ready.game_id.clone(),
            pref: partial,
        },
        _ => GameColorPref::NotReady,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    struct SGNotReady;
    struct SGReady(pub GameReady);

    struct PrefsNone;
    struct PrefsOne(pub SessionColorPref);
    struct PrefsTwo(pub SessionColorPref, pub SessionColorPref);

    impl GameReadyRepo for SGNotReady {
        fn get(&self, _: &SessionId) -> Result<Option<GameReady>, FetchErr> {
            Ok(None)
        }

        fn put(&self, _: GameReady) -> Result<(), WriteErr> {
            panic!()
        }
    }

    impl GameReadyRepo for SGReady {
        fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr> {
            if session_id == &self.0.sessions.0 {
                Ok(Some(self.0.clone()))
            } else if session_id == &self.0.sessions.1 {
                Ok(Some(self.0.clone()))
            } else {
                Ok(None)
            }
        }

        fn put(&self, _: GameReady) -> Result<(), WriteErr> {
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

    #[test]
    fn test_by_session_id_complete() {
        let sid = SessionId::random();
        let cid = ClientId::random();
        let gid = GameId::random();
        let one_pref = SessionColorPref {
            session_id: sid.clone(),
            color_pref: ColorPref::Black,
            client_id: cid.clone(),
        };
        let another_sid = SessionId::random();
        let another_cid = ClientId::random();
        let another_pref = SessionColorPref {
            session_id: another_sid.clone(),
            color_pref: ColorPref::Black,
            client_id: another_cid.clone(),
        };

        let repos = Repos {
            prefs: Rc::new(PrefsTwo(one_pref.clone(), another_pref.clone())),
            game_ready: Rc::new(SGReady(GameReady {
                sessions: (sid.clone(), another_sid.clone()),
                game_id: gid.clone(),
                event_id: EventId::new(),
            })),
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
        let sid = SessionId::random();
        let cid = ClientId::random();
        let gid = GameId::random();
        let pref = SessionColorPref {
            session_id: sid.clone(),
            color_pref: ColorPref::Black,
            client_id: cid.clone(),
        };
        let repos = Repos {
            prefs: Rc::new(PrefsOne(pref.clone())),
            game_ready: Rc::new(SGReady(GameReady {
                sessions: (sid.clone(), SessionId::random()),
                game_id: gid.clone(),
                event_id: EventId::new(),
            })),
        };

        let actual = by_session_id(&sid, &repos).expect("ok");
        assert_eq!(actual, GameColorPref::Partial { game_id: gid, pref })
    }
    #[test]
    fn test_by_session_id_not_ready() {
        let sid = SessionId::random();
        let cid = ClientId::random();
        let repos = Repos {
            prefs: Rc::new(PrefsOne(SessionColorPref {
                session_id: sid.clone(),
                color_pref: ColorPref::Black,
                client_id: cid.clone(),
            })),
            game_ready: Rc::new(SGNotReady),
        };

        let actual = by_session_id(&sid, &repos).expect("ok");
        assert_eq!(actual, GameColorPref::NotReady)
    }

    #[test]
    fn test_by_game_ready_two_prefs() {
        let sid = SessionId::random();
        let gid = GameId::random();

        let another_sid = SessionId::random();
        let sessions = (sid.clone(), another_sid.clone());

        let game_ready = GameReady {
            sessions: sessions.clone(),
            game_id: gid.clone(),
            event_id: EventId::new(),
        };

        let first_pref = SessionColorPref {
            session_id: sid.clone(),
            color_pref: ColorPref::Black,
            client_id: ClientId::random(),
        };
        let second_pref = SessionColorPref {
            session_id: another_sid.clone(),
            color_pref: ColorPref::Black,
            client_id: ClientId::random(),
        };

        let repos = Repos {
            prefs: Rc::new(PrefsTwo(first_pref.clone(), second_pref.clone())),
            game_ready: Rc::new(SGReady(game_ready.clone())),
        };

        let actual = by_game_ready(&game_ready, &repos).expect("ok");
        assert_eq!(
            actual,
            GameColorPref::Complete {
                game_id: gid,
                prefs: (first_pref, second_pref)
            }
        )
    }
    #[test]
    fn test_by_game_ready_one_pref() {
        let sid = SessionId::random();
        let gid = GameId::random();

        let sessions = (sid.clone(), SessionId::random());

        let game_ready = GameReady {
            sessions: sessions.clone(),
            game_id: gid.clone(),
            event_id: EventId::new(),
        };

        let pref = SessionColorPref {
            session_id: sid.clone(),
            color_pref: ColorPref::Black,
            client_id: ClientId::random(),
        };

        let repos = Repos {
            prefs: Rc::new(PrefsOne(pref.clone())),
            game_ready: Rc::new(SGReady(game_ready.clone())),
        };

        let actual = by_game_ready(&game_ready, &repos).expect("ok");
        assert_eq!(actual, GameColorPref::Partial { game_id: gid, pref })
    }
    #[test]
    fn test_by_game_ready_no_prefs() {
        let sid = SessionId::random();
        let gid = GameId::random();

        let sessions = (sid, SessionId::random());

        let game_ready = GameReady {
            sessions: sessions.clone(),
            game_id: gid,
            event_id: EventId::new(),
        };

        let repos = Repos {
            prefs: Rc::new(PrefsNone),
            game_ready: Rc::new(SGReady(game_ready.clone())),
        };

        let actual = by_game_ready(&game_ready, &repos).expect("ok");
        assert_eq!(actual, GameColorPref::NotReady)
    }
}
