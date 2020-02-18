extern crate micro_judge;

use micro_judge::{conn_pool, stream};

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);
    stream::process(conn_pool::create())
}
