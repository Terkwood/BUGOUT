extern crate gateway;

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;

use gateway::backend::repo::{ClientBackendRepo, RedisClientBackendRepo};
use gateway::backend::BackendInitOptions;
use gateway::backend_commands::SessionCommand;
use gateway::backend_events::{BackendEvents, KafkaShutdownEvent};
use gateway::idle_status::{IdleStatusResponse, KafkaActivityObserved, RequestIdleStatus};

use gateway::router::RouterCommand;
use gateway::websocket::WsSession;
use gateway::{backend, env, idle_status, router};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);

    env::init();

    let (session_commands_in, session_commands_out): (
        Sender<SessionCommand>,
        Receiver<SessionCommand>,
    ) = unbounded();

    let (backend_events_in, backend_events_out): (Sender<BackendEvents>, Receiver<BackendEvents>) =
        unbounded();

    let (router_commands_in, router_commands_out): (
        Sender<RouterCommand>,
        Receiver<RouterCommand>,
    ) = unbounded();

    let (shutdown_in, shutdown_out): (Sender<KafkaShutdownEvent>, Receiver<KafkaShutdownEvent>) =
        unbounded();

    let (req_idle_in, req_idle_out): (Sender<RequestIdleStatus>, Receiver<RequestIdleStatus>) =
        unbounded();

    let (idle_resp_in, idle_resp_out): (Sender<IdleStatusResponse>, Receiver<IdleStatusResponse>) =
        unbounded();

    let (kafka_activity_in, kafka_activity_out): (
        Sender<KafkaActivityObserved>,
        Receiver<KafkaActivityObserved>,
    ) = unbounded();

    idle_status::start_monitor(idle_resp_in, shutdown_out, req_idle_out, kafka_activity_out);

    router::start(router_commands_out, backend_events_out, idle_resp_out);

    std::thread::spawn(move || {
        ws::listen("0.0.0.0:3012", |ws_out| {
            WsSession::new(
                ws_out,
                session_commands_in.clone(),
                router_commands_in.clone(),
                req_idle_in.clone(),
            )
        })
        .unwrap()
    });

    // TODO move this
    let client_repo: Box<dyn ClientBackendRepo> = Box::new(RedisClientBackendRepo {
        key_provider: gateway::redis_io::KeyProvider::default(),
        pool: todo!(),
    });
    backend::start_all(BackendInitOptions {
        backend_events_in,
        client_repo,
        kafka_activity_in,
        session_commands_out,
        shutdown_in,
    })
}
