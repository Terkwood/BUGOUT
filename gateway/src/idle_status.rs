use crate::kafka_events::*;
use crate::model::ClientId;
use chrono::{DateTime, Utc};
use crossbeam::{Receiver, Sender};
use crossbeam_channel::select;
use serde_derive::{Deserialize, Serialize};
use std::thread;

/// The running status of an expensive container host
///
/// - Idle (since when)
/// - Booting (since when)
/// - Awake (you may proceed to have fun)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum IdleStatus {
    Idle(DateTime<Utc>),
    Booting(DateTime<Utc>),
    Online,
}

pub struct RequestIdleStatus(pub ClientId);
pub struct IdleStatusResponse(pub ClientId, pub IdleStatus);

pub fn start_monitor(
    status_resp_in: Sender<IdleStatusResponse>,
    shutdown_out: Receiver<ShutdownEvent>,
    req_status_out: Receiver<RequestIdleStatus>,
    kafka_out: Receiver<KafkaEvents>,
) {
    thread::spawn(move || {
        let mut status = IdleStatus::Idle(Utc::now());

        loop {
            select! {
                recv(kafka_out) -> kafka_event =>
                    if let Ok(_) = kafka_event {
                        unimplemented!()
                    } else {
                        println!("err in idle recv for kafka")
                    },
                recv(req_status_out) -> req =>
                    if let Ok(RequestIdleStatus(client_id)) = req {
                        if let Err(e) = status_resp_in.send(IdleStatusResponse(client_id, status)) {
                        println!("err sending idle status resp {}", e)
                    }} else {
                        println!("err on idle recv req status")
                    },
                recv(shutdown_out) -> msg =>
                    if let Ok(_) = msg {
                        println!(" ..SHUTDOWN EVENT DETECTED.. ");
                        status = IdleStatus::Idle(Utc::now());
                    } else {
                        println!("...HALP err on recv shutdown...")
                    }
            }
        }
    });
}
