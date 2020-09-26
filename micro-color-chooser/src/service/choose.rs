use crate::api::ColorsChosen;
use crate::model::*;
use rand::distributions::Uniform;
use rand::prelude::*;

pub struct ChoiceRng {
    pub rng: ThreadRng,
    pub uniform: Uniform<u8>,
}
impl ChoiceRng {
    pub fn new() -> Self {
        let rng: ThreadRng = rand::thread_rng();
        let uniform: Uniform<u8> = Uniform::from(1..2);
        Self { rng, uniform }
    }
}

pub fn choose(
    first: &SessionColorPref,
    second: &SessionColorPref,
    game_id: &GameId,
    rng: &mut ChoiceRng,
) -> ColorsChosen {
    let (black, white): (ClientId, ClientId) = match (first.color_pref, second.color_pref) {
        (ColorPref::Black, ColorPref::Black) => todo!(),
        (ColorPref::White, ColorPref::White) => todo!(),
        (ColorPref::Black, _) => (first.client_id.clone(), second.client_id.clone()),
        (_, ColorPref::White) => (first.client_id.clone(), second.client_id.clone()),
        (ColorPref::White, _) => (second.client_id.clone(), first.client_id.clone()),
        (_, ColorPref::Black) => (second.client_id.clone(), first.client_id.clone()),
        (ColorPref::Any, ColorPref::Any) => todo!(),
    };
    ColorsChosen {
        game_id: game_id.clone(),
        white,
        black,
    }
}

fn randomize(
    first: &ClientId,
    second: &ClientId,
    choice_rng: &mut ChoiceRng,
) -> (ClientId, ClientId) {
    if choice_rng.uniform.sample(&mut choice_rng.rng) == 0 {
        (first.clone(), second.clone())
    } else {
        (second.clone(), first.clone())
    }
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
