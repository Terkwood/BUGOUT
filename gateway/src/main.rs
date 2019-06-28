/// Adapted from https://github.com/housleyjk/ws-rs/blob/master/examples/pong.rs
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate env_logger;
extern crate mio_extras;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate time;
/// An example demonstrating how to send and recieve a custom ping/pong frame.
extern crate ws;

mod kafka;
pub mod model;
mod websocket;

use crossbeam_channel::bounded;
use ws::listen;

use model::Message;
use websocket::WsSession;

fn main() {
    env_logger::init();

    let (kafka_in, kafka_out): (crossbeam::Sender<Message>, crossbeam::Receiver<Message>) =
        bounded(100);
    let (_, router_out): (crossbeam::Sender<Message>, crossbeam::Receiver<Message>) = bounded(100);

    // Run the WebSocket
    listen("127.0.0.1:3012", |out| WsSession {
        client_id: uuid::Uuid::new_v4(),
        out,
        ping_timeout: None,
        expire_timeout: None,
    })
    .unwrap();
}
