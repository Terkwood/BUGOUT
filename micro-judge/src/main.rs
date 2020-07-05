extern crate log;
extern crate micro_judge;

use log::info;

use micro_judge::io::stream;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {:<8} {}", NAME, VERSION);
    let process_opts = stream::StreamOpts::default();
    stream::create_consumer_groups(&process_opts.topics, &process_opts.client);
    stream::process(process_opts);
}
