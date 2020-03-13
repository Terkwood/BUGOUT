#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct KataGoQuery {
    pub id: Id,
    #[serde(rename = "initialStone")]
    pub initial_stones: Vec<Move>,
    pub moves: Vec<Move>,
    pub rules: Rules,
    pub komi: Komi,
    #[serde(rename = "boardXSize")]
    pub board_x_size: u16,
    #[serde(rename = "boardYSize")]
    pub board_y_size: u16,
}

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd)]
pub struct KataGoResponse {
    pub id: Id,
    #[serde(rename = "moveInfos")]
    pub move_infos: Vec<MoveInfo>,
    #[serde(rename = "turnNumber")]
    pub turn_number: u32,
}
