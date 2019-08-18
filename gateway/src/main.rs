extern crate gateway;

use crossbeam_channel::{unbounded, Receiver, Sender};

use gateway::model::{Events, KafkaCommands, KafkaEvents};
use gateway::router::RouterCommand;
use gateway::websocket::WsSession;
use gateway::{kafka, router};

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);

    let (kafka_commands_in, kafka_commands_out): (
        Sender<KafkaCommands>,
        Receiver<KafkaCommands>,
    ) = unbounded();

    let (kafka_events_in, _): (Sender<KafkaEvents>, Receiver<KafkaEvents>) = unbounded();
    let (_, client_events_out): (Sender<Events>, Receiver<Events>) = unbounded();

    let (router_commands_in, router_commands_out): (
        Sender<RouterCommand>,
        Receiver<RouterCommand>,
    ) = unbounded();

    kafka::start(
        kafka_events_in,
        router_commands_in.clone(),
        kafka_commands_out,
    );

    router::start(router_commands_out, client_events_out);

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
