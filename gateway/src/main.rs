#![feature(bind_by_move_pattern_guards)]
#![feature(checked_duration_since)]
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate envy;
extern crate harsh;
#[macro_use]
extern crate lazy_static;
extern crate mio_extras;
extern crate rand;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate ws;

mod compact_ids;
mod env;
mod json;
mod kafka;
mod logging;
mod model;
mod router;
mod topics;
mod websocket;

use crossbeam_channel::{unbounded, Receiver, Sender};

use model::{Events, KafkaCommands};
use router::RouterCommand;
use websocket::WsSession;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);

    let (kafka_commands_in, kafka_commands_out): (
        Sender<KafkaCommands>,
        Receiver<KafkaCommands>,
    ) = unbounded();

    let (kafka_events_in, kafka_events_out): (Sender<Events>, Receiver<Events>) = unbounded();

    let (router_commands_in, router_commands_out): (
        Sender<RouterCommand>,
        Receiver<RouterCommand>,
    ) = unbounded();

    kafka::start(
        kafka_events_in,
        router_commands_in.clone(),
        kafka_commands_out,
    );

    router::start(router_commands_out, kafka_events_out);

    ws::listen("0.0.0.0:3012", |ws_out| {
        WsSession::new(
            uuid::Uuid::new_v4(),
            ws_out,
            kafka_commands_in.clone(),
            router_commands_in.clone(),
        )
    })
    .unwrap();
}
