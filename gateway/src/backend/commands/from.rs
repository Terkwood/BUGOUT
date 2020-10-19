use crate::backend::commands::*;
use crate::model::{ColorPref, Coord, Move, Player, Visibility};

use color_model as color;
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

impl From<Visibility> for lobby::Visibility {
    fn from(v: Visibility) -> Self {
        match v {
            Visibility::Private => lobby::Visibility::Private,
            Visibility::Public => lobby::Visibility::Public,
        }
    }
}

impl From<Coord> for moves::Coord {
    fn from(c: Coord) -> Self {
        moves::Coord { x: c.x, y: c.y }
    }
}

impl From<Move> for sync::Move {
    fn from(m: Move) -> Self {
        sync::Move {
            player: m.player.into(),
            turn: m.turn as u32,
            coord: m.coord.map(|c| c.into()),
        }
    }
}
