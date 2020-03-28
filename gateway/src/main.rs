extern crate gateway;

use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::executor::block_on;

use gateway::env;
use gateway::idle_status;
use gateway::idle_status::{IdleStatusResponse, KafkaActivityObserved, RequestIdleStatus};
use gateway::kafka_commands::KafkaCommands;
use gateway::kafka_events::{KafkaEvents, ShutdownEvent};
use gateway::router::RouterCommand;
use gateway::websocket::WsSession;
use gateway::{kafka_io, router};

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);

    env_logger::init();
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

    let (kafka_activity_in, kafka_activity_out): (
        Sender<KafkaActivityObserved>,
        Receiver<KafkaActivityObserved>,
    ) = unbounded();

    idle_status::start_monitor(idle_resp_in, shutdown_out, req_idle_out, kafka_activity_out);

    router::start(router_commands_out, kafka_events_out, idle_resp_out);

    std::thread::spawn(move || {
        ws::listen("0.0.0.0:3012", |ws_out| {
            WsSession::new(
                ws_out,
                kafka_commands_in.clone(),
                router_commands_in.clone(),
                req_idle_in.clone(),
            )
        })
        .unwrap()
    });

    block_on(kafka_io::start(
        kafka_events_in,
        shutdown_in,
        kafka_activity_in,
        kafka_commands_out,
    ))
}
