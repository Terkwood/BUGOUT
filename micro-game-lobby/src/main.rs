extern crate micro_game_lobby;
use micro_game_lobby::*;

use components::Components;

fn main() {
    stream::process(&Components::default())
}
