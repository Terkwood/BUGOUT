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

pub fn start_monitor(shutdown_out: crossbeam::Receiver<ShutdownEvent>) {
    thread::spawn(move || {
        // Please Watch For shutdown event

        loop {
            // Block on this channel, since there won't
            // be any activity for a long time
            let msg = shutdown_out.recv();
            if let Ok(_) = msg {
                println!(" ..SHUTDOWN EVENT DETECTED.. ");
                unimplemented!()
            } else {
                println!("...HALP...")
            }
        }
    });
}
