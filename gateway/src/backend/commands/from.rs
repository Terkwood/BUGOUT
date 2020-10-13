use crate::backend::commands::*;
use crate::model::{ColorPref, Coord, Move, Player, ProvideHistoryCommand, Visibility};
use uuid::Uuid;

use color_model as color;
use core_model as core;
use lobby_model as lobby;
use move_model as moves;
use sync_model as sync;

impl From<crate::model::Player> for moves::Player {
    fn from(p: crate::model::Player) -> Self {
        match p {
            Player::BLACK => moves::Player::BLACK,
            Player::WHITE => moves::Player::WHITE,
        }
    }
}

impl From<ChooseColorPrefBackendCommand> for color::api::ChooseColorPref {
    fn from(c: ChooseColorPrefBackendCommand) -> Self {
        color::api::ChooseColorPref {
            client_id: c.client_id.into_shared(),
            color_pref: c.color_pref.into(),
            session_id: c.session_id.into_shared(),
        }
    }
}
impl From<ColorPref> for color::ColorPref {
    fn from(c: ColorPref) -> Self {
        match c {
            ColorPref::Black => color::ColorPref::Black,
            ColorPref::White => color::ColorPref::White,
            ColorPref::Any => color::ColorPref::Any,
        }
    }
}
