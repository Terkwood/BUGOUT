extern crate bugle;

use bugle::{env, subscriber};

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    env::init();
    println!("🔢 {:<8} {}", NAME, VERSION);
    subscriber::start()
}
