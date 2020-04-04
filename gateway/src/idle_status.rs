use crate::backend_events::*;
use crate::model::ClientId;
use crate::redis_io::RedisPool;
use crate::wakeup::RedisWakeup;

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
#[serde(tag = "status")] // Avoid conflict with ClientEvents tag
pub enum IdleStatus {
    Idle { since: DateTime<Utc> },
    Booting { since: DateTime<Utc> },
    Online,
}

pub struct RequestIdleStatus(pub ClientId);
pub struct IdleStatusResponse(pub ClientId, pub IdleStatus);

pub struct KafkaActivityObserved;

pub fn start_monitor(
    status_resp_in: Sender<IdleStatusResponse>,
    shutdown_out: Receiver<KafkaShutdownEvent>,
    req_status_out: Receiver<RequestIdleStatus>,
    kafka_out: Receiver<KafkaActivityObserved>,
    pool: &RedisPool,
) {
    let pc = pool.clone();
    thread::spawn(move || {
        let mut status = IdleStatus::Idle { since: Utc::now() };
        let redis_wakeup = RedisWakeup::new(&pc);

        loop {
            select! {
                recv(kafka_out) -> kafka_event =>
                    if let Ok(_) = kafka_event {
                        match status {
                            IdleStatus::Online => (),
                            _ => {
                                status = IdleStatus::Online
                            },
                        }
                    } else {
                        println!("err in idle recv for kafka")
                    },
                recv(req_status_out) -> req =>
                    if let Ok(RequestIdleStatus(client_id)) = req {
                        if let Err(e) = status_resp_in.send(IdleStatusResponse(client_id, status)) {
                            println!("err sending idle status resp {}", e)
                        }

                        match status {
                            IdleStatus::Online => (),
                            _ => {
                                if let Err(e) = redis_wakeup.publish(client_id)
                                {
                                    println!("error publishing wakeup to redis {}", e)
                                } else if let IdleStatus::Idle{since: _} = status {
                                    status = IdleStatus::Booting{since: Utc::now()}
                                }
                            },
                        }
                } else {
                        println!("err on idle recv req status")
                    },
                recv(shutdown_out) -> msg =>
                    if let Ok(_) = msg {
                        println!("☠️ SHUTDOWN");
                        status = IdleStatus::Idle { since: Utc::now() };
                    } else {
                        println!("...HALP err on recv shutdown...")
                    }
            }
        }
    });
}
