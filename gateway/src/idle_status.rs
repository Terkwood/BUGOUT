use crate::kafka_events::*;
use chrono::{DateTime, Utc};
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

pub fn start_monitor(kafka_out: crossbeam::Receiver<ShutdownEvent>) {
    thread::spawn(move || {
        // Please Watch For shutdown event

        loop {
            select! {
            recv(kafka_out) -> event =>
                match event {
                    Ok(_) => {
                        println!(" ..SHUTDOWN EVENT DETECTED.. ");
                        unimplemented!()
                    },
                    Err(e) => println!("Error reading kafka event in idle monitor: {:?}", e),
                }
            }
        }
    });
}
