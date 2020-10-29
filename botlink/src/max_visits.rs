use bot_model::Difficulty;

/// Lower than 2 and you'll see an error
const EASY: u16 = 2;
const MEDIUM: u16 = 5;
const HARD: u16 = 100;

pub fn convert(difficulty: Difficulty) -> Option<u16> {
    match difficulty {
        Difficulty::Easy => Some(EASY),
        Difficulty::Medium => Some(MEDIUM),
        Difficulty::Hard => Some(HARD),
        Difficulty::Max => None,
    }
}
