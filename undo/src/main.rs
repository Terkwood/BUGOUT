use log::info;
use undo::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = redis_client();
    let components = Components::new(client.clone());
    stream::init(&client, components);
}
