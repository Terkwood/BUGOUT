use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::backend::commands::BackendCommands;
use crate::backend::events::BackendEvents;
use crate::idle_status::{IdleStatusResponse, RequestIdleStatus};
use crate::router::RouterCommand;

#[derive(Clone)]
pub struct MainChannels {
    pub session_commands_in: Sender<BackendCommands>,
    pub session_commands_out: Receiver<BackendCommands>,
    pub backend_events_in: Sender<BackendEvents>,
    pub backend_events_out: Receiver<BackendEvents>,
    pub router_commands_in: Sender<RouterCommand>,
    pub router_commands_out: Receiver<RouterCommand>,
    pub req_idle_in: Sender<RequestIdleStatus>,
    pub req_idle_out: Receiver<RequestIdleStatus>,
    pub idle_resp_in: Sender<IdleStatusResponse>,
    pub idle_resp_out: Receiver<IdleStatusResponse>,
}

impl MainChannels {
    pub fn create() -> Self {
        let (session_commands_in, session_commands_out): (
            Sender<BackendCommands>,
            Receiver<BackendCommands>,
        ) = unbounded();

        let (backend_events_in, backend_events_out): (
            Sender<BackendEvents>,
            Receiver<BackendEvents>,
        ) = unbounded();

        let (router_commands_in, router_commands_out): (
            Sender<RouterCommand>,
            Receiver<RouterCommand>,
        ) = unbounded();

        let (req_idle_in, req_idle_out): (Sender<RequestIdleStatus>, Receiver<RequestIdleStatus>) =
            unbounded();

        let (idle_resp_in, idle_resp_out): (
            Sender<IdleStatusResponse>,
            Receiver<IdleStatusResponse>,
        ) = unbounded();

        MainChannels {
            idle_resp_in,
            idle_resp_out,
            req_idle_in,
            req_idle_out,
            backend_events_in,
            backend_events_out,
            router_commands_in,
            router_commands_out,
            session_commands_in,
            session_commands_out,
        }
    }
}
