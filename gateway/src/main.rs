/// Adapted from https://github.com/housleyjk/ws-rs/blob/master/examples/pong.rs
extern crate env_logger;
extern crate mio_extras;
extern crate serde;
extern crate time;
/// An example demonstrating how to send and recieve a custom ping/pong frame.
extern crate ws;

pub mod model;
mod server;

use ws::listen;

use server::Server;

fn main() {
    // Setup logging
    env_logger::init();

    // Run the WebSocket
    listen("127.0.0.1:3012", |out| Server {
        out,
        ping_timeout: None,
        expire_timeout: None,
    })
    .unwrap();
}
