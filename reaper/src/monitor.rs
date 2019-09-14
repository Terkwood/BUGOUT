use crossbeam_channel::select;

use crate::model::*;

/// Start a process which
pub fn start(
    shutdown_in: crossbeam::Sender<ShutdownCommand>,
    activity_out: crossbeam::Receiver<KafkaActivity>,
) {
    let mut monitor = Monitor::new();

    loop {
        select! {
            recv(activity_out) -> command =>
                match command {
                    Ok(k) => monitor.push(k),
                    Err(e) => println!("Failed to select in monitor: {:?}", e),
                }
        }
    }
}

struct Monitor(Vec<KafkaActivity>);
impl Monitor {
    pub fn new() -> Monitor {
        Monitor(vec![])
    }

    pub fn push(&mut self, k: KafkaActivity) {
        self.0.push(k)
    }

    pub fn is_system_dead_enough(&mut self) -> bool {
        self.prune();

        unimplemented!()
    }

    fn prune(&mut self) {}
}
