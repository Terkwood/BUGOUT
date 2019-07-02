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

use model::Commands;
use websocket::WsSession;

fn main() {
    let (commands_in, commands_out): (
        crossbeam::Sender<Commands>,
        crossbeam::Receiver<Commands>,
    ) = unbounded();

    kafka::start(commands_out);

    ws::listen("0.0.0.0:3012", |ws_out| {
        WsSession::new(uuid::Uuid::new_v4(), ws_out, commands_in.clone())
    })
    .unwrap();
}
