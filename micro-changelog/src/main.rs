const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use micro_changelog::stream;
use micro_changelog::Components;
use stream::StreamTopics;

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);
    stream::process(StreamTopics::default(), &Components::default())
}
