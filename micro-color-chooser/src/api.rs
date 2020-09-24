use crate::model::*;
pub struct ChooseColorPref {}
pub struct ColorsChosen {
    pub game_id: GameId,
    pub black: ClientId,
    pub white: ClientId,
}
