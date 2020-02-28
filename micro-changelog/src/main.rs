const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use micro_changelog;

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);
}
