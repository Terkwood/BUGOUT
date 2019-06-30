extern crate crossbeam;
extern crate crossbeam_channel;
extern crate mio_extras;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate ws;

mod kafka;
pub mod model;
mod websocket;

use crossbeam_channel::unbounded;

use model::BugoutMessage;
use websocket::WsSession;

fn main() {
    let (kafka_in, _kafka_out): (
        crossbeam::Sender<BugoutMessage>,
        crossbeam::Receiver<BugoutMessage>,
    ) = unbounded();

    let (router_in, router_out): (
        crossbeam::Sender<BugoutMessage>,
        crossbeam::Receiver<BugoutMessage>,
    ) = unbounded();

    kafka::start(router_in);

    ws::listen("0.0.0.0:3012", |out| WsSession {
        client_id: uuid::Uuid::new_v4(),
        out,
        ping_timeout: None,
        expire_timeout: None,
        kafka_in: kafka_in.clone(),
        router_out: router_out.clone(),
    })
    .unwrap();
}
