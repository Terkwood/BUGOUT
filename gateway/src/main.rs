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
mod router;
mod websocket;

use crossbeam_channel::{unbounded, Receiver, Sender};

use model::{Commands, Events};
use websocket::WsSession;

fn main() {
    let (commands_in, commands_out): (Sender<Commands>, Receiver<Commands>) = unbounded();

    let (events_in, events_out): (Sender<Events>, Receiver<Events>) = unbounded();

    kafka::start(events_in, commands_out);

    // TODO routing and such

    ws::listen("0.0.0.0:3012", |ws_out| {
        WsSession::new(
            uuid::Uuid::new_v4(),
            ws_out,
            commands_in.clone(),
            events_out.clone(),
        )
    })
    .unwrap();
}
