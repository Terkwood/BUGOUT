use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::backend_commands::BackendCommands;
use crate::backend_events::{BackendEvents, KafkaShutdownEvent};
use crate::idle_status::{IdleStatusResponse, KafkaActivityObserved, RequestIdleStatus};
use crate::router::RouterCommand;

#[derive(Clone)]
pub struct MainChannels {
    pub session_commands_in: Sender<BackendCommands>,
    pub session_commands_out: Receiver<BackendCommands>,
    pub backend_events_in: Sender<BackendEvents>,
    pub backend_events_out: Receiver<BackendEvents>,
    pub router_commands_in: Sender<RouterCommand>,
    pub router_commands_out: Receiver<RouterCommand>,
    pub shutdown_in: Sender<KafkaShutdownEvent>,
    pub shutdown_out: Receiver<KafkaShutdownEvent>,
    pub req_idle_in: Sender<RequestIdleStatus>,
    pub req_idle_out: Receiver<RequestIdleStatus>,
    pub idle_resp_in: Sender<IdleStatusResponse>,
    pub idle_resp_out: Receiver<IdleStatusResponse>,
    pub kafka_activity_in: Sender<KafkaActivityObserved>,
    pub kafka_activity_out: Receiver<KafkaActivityObserved>,
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

        let (shutdown_in, shutdown_out): (
            Sender<KafkaShutdownEvent>,
            Receiver<KafkaShutdownEvent>,
        ) = unbounded();

        let (req_idle_in, req_idle_out): (Sender<RequestIdleStatus>, Receiver<RequestIdleStatus>) =
            unbounded();

        let (idle_resp_in, idle_resp_out): (
            Sender<IdleStatusResponse>,
            Receiver<IdleStatusResponse>,
        ) = unbounded();

        let (kafka_activity_in, kafka_activity_out): (
            Sender<KafkaActivityObserved>,
            Receiver<KafkaActivityObserved>,
        ) = unbounded();

        MainChannels {
            kafka_activity_in,
            kafka_activity_out,
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
            shutdown_in,
            shutdown_out,
        }
    }
}
