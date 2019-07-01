extern crate crossbeam;
extern crate crossbeam_channel;
extern crate mio_extras;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate ws;

mod json;
mod kafka;
pub mod model;
mod websocket;

use crossbeam_channel::unbounded;
use std::thread;

use model::BugoutMessage;
use websocket::WsSession;

fn main() {
    let (kafka_in, kafka_out): (
        crossbeam::Sender<BugoutMessage>,
        crossbeam::Receiver<BugoutMessage>,
    ) = unbounded();

    let kic = kafka_in.clone();
    thread::spawn(move || {
        kafka::start(kic);
    });

    ws::listen("0.0.0.0:3012", |ws_out| {
        WsSession::new(
            uuid::Uuid::new_v4(),
            ws_out,
            kafka_in.clone(),
            kafka_out.clone(),
        )
    })
    .unwrap();
}
