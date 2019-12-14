extern crate bugle;

use bugle::subscriber;

use serde_json;
use uuid::Uuid;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);
    println!(
        "📯 {}",
        serde_json::to_string(&bugle::WakeUpEvent {
            client_id: Uuid::new_v4()
        })
        .unwrap()
    );
    subscriber::start()
}
