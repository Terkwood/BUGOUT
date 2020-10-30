use bot_model::Bot;

/// Lower than 2 and you'll see an error
const INSTANT: u16 = 2;

pub fn convert(bot: Bot) -> Option<u16> {
    match bot {
        Bot::KataGoInstant => Some(INSTANT),
        Bot::KataGoFullStrength => None,
    }
}
