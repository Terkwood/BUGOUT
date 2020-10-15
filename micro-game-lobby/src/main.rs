use components::Components;
use log::info;
use micro_game_lobby::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    stream::process(&Components::default())
}
