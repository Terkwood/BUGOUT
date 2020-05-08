extern crate micro_game_lobby;
use micro_game_lobby::*;

use components::Components;
use topics::StreamTopics;

fn main() {
    stream::process(&StreamTopics::default(), &Components::default())
}
