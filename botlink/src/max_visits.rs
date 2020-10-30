use bot_model::Bot;

/// Lower than 2 and you'll see an error
const INSTANT: u16 = 2;

pub fn convert(difficulty: Bot) -> Option<u16> {
    match difficulty {
        Bot::KataGoInstant => Some(INSTANT),
        Bot::KataGoFullStrength => None,
    }
}
