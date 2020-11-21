use bot_model::Bot;

/// Lower than 2 and you'll see an error
const ONE_STAR_VISITS: u16 = 2;
const TWO_STAR_VISITS: u16 = 166;
const THREE_STAR_VISITS: u16 = 333;

pub fn max_visits(bot: Bot) -> Option<u16> {
    match bot {
        Bot::KataGoOneStar => Some(ONE_STAR_VISITS),
        Bot::KataGoTwoStars => Some(TWO_STAR_VISITS),
        Bot::KataGoThreeStars => Some(THREE_STAR_VISITS),
        Bot::KataGoFourStars => None, // give it everything you've got (~500 visits)
    }
}
