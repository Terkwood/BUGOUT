extern crate dotenv;
#[macro_use]
extern crate lazy_static;

mod env;
mod shutdown;

use shutdown::shutdown;

fn main() {
    env::init();
    shutdown();
}
