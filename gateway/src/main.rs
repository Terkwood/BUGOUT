extern crate gateway;
use log::info;

use gateway::backend::BackendInitOptions;
use gateway::channels::MainChannels;
use gateway::redis_io;
use gateway::websocket::WsSession;
use gateway::{backend, env, idle_status, router};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);

    env::init();
    let mc = MainChannels::create();
    let pool = redis_io::create_pool();
    idle_status::start_monitor(
        mc.idle_resp_in,
        mc.shutdown_out,
        mc.req_idle_out,
        mc.kafka_activity_out,
        &pool,
    );

    router::start(
        mc.router_commands_out,
        mc.backend_events_out,
        mc.idle_resp_out,
    );

    let sci = mc.session_commands_in;
    let rci = mc.router_commands_in;
    let rii = mc.req_idle_in;
    std::thread::spawn(move || {
        ws::listen("0.0.0.0:3012", |ws_out| {
            WsSession::new(ws_out, sci.clone(), rci.clone(), rii.clone())
        })
        .unwrap()
    });

    backend::start_all(BackendInitOptions {
        backend_events_in: mc.backend_events_in,
        kafka_activity_in: mc.kafka_activity_in,
        session_commands_out: mc.session_commands_out,
        shutdown_in: mc.shutdown_in,
        redis_pool: pool,
    })
}
