extern crate log;
extern crate micro_judge;

use log::info;

use micro_judge::io::{conn_pool, stream};

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {:<8} {}", NAME, VERSION);
    stream::process(
        stream::ProcessOpts::default(),
        &conn_pool::create(conn_pool::RedisHostUrl::default()),
    );
}
