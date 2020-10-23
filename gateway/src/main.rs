extern crate gateway;
use gateway::channels::MainChannels;
use gateway::redis_io;
use gateway::websocket::WsSession;
use gateway::{backend, env, idle_status, router};
use log::info;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);

    env::init();
    let mc = MainChannels::create();
    let client = redis_io::create_redis_client();
    idle_status::start_monitor(mc.idle_resp_in.clone(), mc.req_idle_out.clone());

    router::start(
        mc.router_commands_out.clone(),
        mc.backend_events_out.clone(),
        mc.idle_resp_out.clone(),
    );

    let sci = mc.session_commands_in.clone();
    let rci = mc.router_commands_in.clone();
    let rii = mc.req_idle_in.clone();
    std::thread::spawn(move || {
        ws::listen("0.0.0.0:3012", |ws_out| {
            WsSession::new(ws_out, sci.clone(), rci.clone(), rii.clone())
        })
        .unwrap()
    });

    backend::start(&mc, client)
}
