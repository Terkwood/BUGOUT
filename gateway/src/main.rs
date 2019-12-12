extern crate gateway;

use crossbeam_channel::{unbounded, Receiver, Sender};

use gateway::env;
use gateway::idle_status;
use gateway::idle_status::{IdleStatusResponse, RequestIdleStatus};
use gateway::kafka_commands::KafkaCommands;
use gateway::kafka_events::{KafkaEvents, ShutdownEvent};
use gateway::router::RouterCommand;
use gateway::websocket::WsSession;
use gateway::{kafka_io, router};

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);

    env::init();

    let (kafka_commands_in, kafka_commands_out): (Sender<KafkaCommands>, Receiver<KafkaCommands>) =
        unbounded();

    let (kafka_events_in, kafka_events_out): (Sender<KafkaEvents>, Receiver<KafkaEvents>) =
        unbounded();

    let (router_commands_in, router_commands_out): (
        Sender<RouterCommand>,
        Receiver<RouterCommand>,
    ) = unbounded();

    let (shutdown_in, shutdown_out): (Sender<ShutdownEvent>, Receiver<ShutdownEvent>) = unbounded();

    let (req_idle_in, req_idle_out): (Sender<RequestIdleStatus>, Receiver<RequestIdleStatus>) =
        unbounded();

    let (idle_resp_in, idle_resp_out): (Sender<IdleStatusResponse>, Receiver<IdleStatusResponse>) =
        unbounded();

    kafka_io::start(kafka_events_in, shutdown_in, kafka_commands_out);

    idle_status::start_monitor(idle_resp_in, shutdown_out, req_idle_out);

    router::start(router_commands_out, kafka_events_out, idle_resp_out);

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
