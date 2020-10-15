use micro_game_lobby::*;
use log::info;
use components::Components;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    stream::process(&Components::default())
}
