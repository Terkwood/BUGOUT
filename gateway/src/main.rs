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
    let (bugout_commands_in, bugout_commands_out): (Sender<Commands>, Receiver<Commands>) =
        unbounded();

    let (kafka_events_in, kafka_events_out): (Sender<Events>, Receiver<Events>) = unbounded();

    kafka::start(kafka_events_in, bugout_commands_out);

    let (router_commands_in, router_commands_out): (
        Sender<router::RouterCommand>,
        Receiver<router::RouterCommand>,
    ) = unbounded();

    router::start(router_commands_out, kafka_events_out);

    // TODO routing and such

    ws::listen("0.0.0.0:3012", |ws_out| {
        WsSession::new(
            uuid::Uuid::new_v4(),
            ws_out,
            bugout_commands_in.clone(),
            unimplemented!(),
        )
    })
    .unwrap();
}
