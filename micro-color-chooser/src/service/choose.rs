use crate::api::ColorsChosen;
use crate::model::*;
pub fn choose(
    first: &SessionColorPref,
    second: &SessionColorPref,
    game_id: &GameId,
) -> ColorsChosen {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn no_conflict() {
        let game_id = GameId::random();
        let first_sid = SessionId::random();
        let second_sid = SessionId::random();
        let first_cid = ClientId::random();
        let second_cid = ClientId::random();

        let first_pref = SessionColorPref {
            color_pref: ColorPref::Black,
            session_id: first_sid,
            client_id: first_cid.clone(),
        };
        let second_pref = SessionColorPref {
            color_pref: ColorPref::White,
            session_id: second_sid,
            client_id: second_cid.clone(),
        };
        let actual = choose(&first_pref, &second_pref, &game_id);
        assert_eq!(
            actual,
            ColorsChosen {
                game_id,
                black: first_cid,
                white: second_cid
            }
        )
    }
    #[test]
    pub fn conflict_black() {
        let game_id = GameId::random();
        let first_sid = SessionId::random();
        let second_sid = SessionId::random();
        let first_cid = ClientId::random();
        let second_cid = ClientId::random();

        let first_pref = SessionColorPref {
            color_pref: ColorPref::Black,
            session_id: first_sid,
            client_id: first_cid.clone(),
        };
        let second_pref = SessionColorPref {
            color_pref: ColorPref::Black,
            session_id: second_sid,
            client_id: second_cid.clone(),
        };
        let actual = choose(&first_pref, &second_pref, &game_id);
        assert_ne!(actual.black, actual.white)
    }

    #[test]
    pub fn conflict_white() {
        let game_id = GameId::random();
        let first_sid = SessionId::random();
        let second_sid = SessionId::random();
        let first_cid = ClientId::random();
        let second_cid = ClientId::random();

        let first_pref = SessionColorPref {
            color_pref: ColorPref::White,
            session_id: first_sid,
            client_id: first_cid.clone(),
        };
        let second_pref = SessionColorPref {
            color_pref: ColorPref::White,
            session_id: second_sid,
            client_id: second_cid.clone(),
        };
        let actual = choose(&first_pref, &second_pref, &game_id);
        assert_ne!(actual.black, actual.white)
    }
    #[test]
    pub fn both_any() {
        let game_id = GameId::random();
        let first_sid = SessionId::random();
        let second_sid = SessionId::random();
        let first_cid = ClientId::random();
        let second_cid = ClientId::random();

        let first_pref = SessionColorPref {
            color_pref: ColorPref::Any,
            session_id: first_sid,
            client_id: first_cid.clone(),
        };
        let second_pref = SessionColorPref {
            color_pref: ColorPref::Any,
            session_id: second_sid,
            client_id: second_cid.clone(),
        };
        let actual = choose(&first_pref, &second_pref, &game_id);
        assert_ne!(actual.black, actual.white)
    }

    #[test]
    pub fn first_any() {
        let game_id = GameId::random();
        let first_sid = SessionId::random();
        let second_sid = SessionId::random();
        let first_cid = ClientId::random();
        let second_cid = ClientId::random();

        let first_pref = SessionColorPref {
            color_pref: ColorPref::Any,
            session_id: first_sid,
            client_id: first_cid.clone(),
        };
        let second_pref = SessionColorPref {
            color_pref: ColorPref::Black,
            session_id: second_sid,
            client_id: second_cid.clone(),
        };
        let actual = choose(&first_pref, &second_pref, &game_id);
        assert_eq!(
            actual,
            ColorsChosen {
                game_id,
                black: second_cid,
                white: first_cid
            }
        )
    }

    #[test]
    pub fn second_any() {
        let game_id = GameId::random();
        let first_sid = SessionId::random();
        let second_sid = SessionId::random();
        let first_cid = ClientId::random();
        let second_cid = ClientId::random();

        let first_pref = SessionColorPref {
            color_pref: ColorPref::Black,
            session_id: first_sid,
            client_id: first_cid.clone(),
        };
        let second_pref = SessionColorPref {
            color_pref: ColorPref::Any,
            session_id: second_sid,
            client_id: second_cid.clone(),
        };
        let actual = choose(&first_pref, &second_pref, &game_id);
        assert_eq!(
            actual,
            ColorsChosen {
                game_id,
                black: first_cid,
                white: second_cid
            }
        )
    }
}
