use std::thread;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use crossbeam_channel::{select, tick};

use crate::env::ALLOWED_IDLE_SECS;
use crate::model::*;

const TICK_SECS: u64 = 5;

/// Start a process which
pub fn start(
    shutdown_in: crossbeam::Sender<ShutdownCommand>,
    activity_out: crossbeam::Receiver<KafkaActivity>,
) {
    let mut monitor = Monitor::new();
    let ticker = tick(Duration::from_secs(TICK_SECS));

    thread::spawn(move || loop {
        select! {
            recv(ticker) -> _ => if monitor.is_system_idle() {
                if let Err(e) = shutdown_in.send(ShutdownCommand::new()) {
                    println!("Failed to send shutdown command: {:?}", e)
                }
            },
            recv(activity_out) -> command =>
                match command {
                    Ok(k) => monitor.push(k),
                    Err(e) => println!("Failed to select in monitor: {:?}", e),
                }
        }
    });
}

struct Monitor(Vec<DateTime<Utc>>);
impl Monitor {
    pub fn new() -> Monitor {
        Monitor(vec![])
    }

    pub fn push(&mut self, k: KafkaActivity) {
        self.0.push(Utc.timestamp_millis(k.timestamp))
    }

    pub fn is_system_idle(&mut self) -> bool {
        self.prune();

        self.0.is_empty()
    }

    fn prune(&mut self) {
        // ignore order, but minimal shuffling
        for i in (0..self.0.len()).rev() {
            if is_expired(self.0[i]) {
                self.0.swap_remove(i);
            }
        }
    }
}

fn is_expired(timestamp: DateTime<Utc>) -> bool {
    let since_then = Utc::now().signed_duration_since(timestamp);

    since_then.num_seconds() > *ALLOWED_IDLE_SECS
}
