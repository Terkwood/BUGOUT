use crate::model::ClientId;
use crossbeam_channel::{select, Receiver, Sender};
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::thread;

/// A trivial enum that shows the system is always awake
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
#[serde(tag = "status")] // Avoid conflict with ClientEvents tag
pub enum IdleStatus {
    Online,
}

pub struct RequestIdleStatus(pub ClientId);
pub struct IdleStatusResponse(pub ClientId, pub IdleStatus);

const STATUS: IdleStatus = IdleStatus::Online;

pub fn start_monitor(
    status_resp_in: Sender<IdleStatusResponse>,
    req_status_out: Receiver<RequestIdleStatus>,
) {
    thread::spawn(move || loop {
        select! {
            recv(req_status_out) -> req =>
                if let Ok(RequestIdleStatus(client_id)) = req {
                    if let Err(e) = status_resp_in.send(IdleStatusResponse(client_id, STATUS)) {
                        error!("err sending idle status resp {}", e)
                    }
            } else {
                error!("err on idle recv req status")
            },
        }
    });
}
