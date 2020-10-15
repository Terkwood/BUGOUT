use log::info;
use micro_game_lobby::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = redis_client();
    let components = Components::new(client.clone());
    stream::create_consumer_group(&client);
    stream::process(&components)
}
