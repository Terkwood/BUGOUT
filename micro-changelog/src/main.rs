const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use micro_changelog::repo::game_states::GameStatesRepo;
use micro_changelog::stream;
use stream::StreamTopics;

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);
    stream::process(StreamTopics::default(), GameStatesRepo::default())
}
