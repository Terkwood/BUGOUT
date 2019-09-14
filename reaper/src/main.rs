mod env;
mod shutdown;

use shutdown::shutdown;

fn main() {
    env::init();
    shutdown();
}
