/// Adapted from https://github.com/housleyjk/ws-rs/blob/master/examples/pong.rs
extern crate crossbeam;
extern crate crossbeam_channel;
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

use model::BugoutMessage;
use websocket::WsSession;

fn main() {
    let (kafka_in, _kafka_out): (
        crossbeam::Sender<BugoutMessage>,
        crossbeam::Receiver<BugoutMessage>,
    ) = bounded(100);
    let (router_in, router_out): (
        crossbeam::Sender<BugoutMessage>,
        crossbeam::Receiver<BugoutMessage>,
    ) = bounded(100);

    kafka::consume_and_forward(
        "kafka:9092",
        "gateway",
        &["bugout-make-move-cmd", "bugout-move-made-ev"],
        router_in,
    );

    // Run the WebSocket
    listen("0.0.0.0:3012", |out| WsSession {
        client_id: uuid::Uuid::new_v4(),
        out,
        ping_timeout: None,
        expire_timeout: None,
        kafka_in: kafka_in.clone(),
        router_out: router_out.clone(),
    })
    .unwrap();
}
