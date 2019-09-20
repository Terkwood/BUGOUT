use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crossbeam_channel::{select, tick};

use crate::env::{ALLOWED_IDLE_SECS, DISABLED};
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
                if *DISABLED {
                    println!("SHUTDOWN event ignored at {:#?}", SystemTime::now())
                } else {
                    println!("SHUTDOWN at {:#?}", SystemTime::now());
                    if let Err(e) = shutdown_in.send(ShutdownCommand::new()) {
                        println!("Failed to send shutdown command: {:?}", e)
                    }
                }
            },
            recv(activity_out) -> command =>
                match command {
                    Ok(_) => monitor.observe(),
                    Err(e) => println!("Failed to select in monitor: {:?}", e),
                }
        }
    });
}

struct Monitor(Vec<Instant>);
impl Monitor {
    pub fn new() -> Monitor {
        Monitor(vec![])
    }

    /// Update the monitor to signal that we've
    /// witnessed some type of activity on the system.
    /// We use the current system time as a conservative
    /// measure for
    pub fn observe(&mut self) {
        self.0.push(Instant::now())
    }

    pub fn is_system_idle(&mut self) -> bool {
        self.prune();

        self.0.is_empty()
    }

    fn prune(&mut self) {
        // ignore order, but minimal shuffling
        for i in (0..self.0.len()).rev() {
            if is_expired(self.0[i], *ALLOWED_IDLE_SECS) {
                println!("Expired: {:?}", self.0[i]);
                self.0.swap_remove(i);
            }
        }
    }
}

fn is_expired(instant: Instant, allowed_idle_secs: u64) -> bool {
    let since_then = instant.elapsed().as_secs();

    since_then > allowed_idle_secs
}

#[cfg(test)]
mod tests {
    extern crate time;
    use super::*;

    #[test]
    fn test_is_expired() {
        let then = Instant::now();
        let allowed_secs: u64 = 1;
        std::thread::sleep(Duration::from_secs(allowed_secs * 2));
        assert_eq!(true, is_expired(then, allowed_secs));
    }
}
