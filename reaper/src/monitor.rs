use crossbeam_channel::select;

use crate::model::*;

/// Start a process which
pub fn start(
    shutdown_in: crossbeam::Sender<ShutdownCommand>,
    activity_out: crossbeam::Receiver<KafkaActivity>,
) {
    loop {
        select! {
            recv(activity_out) -> command =>
                match command {
                    Ok(_) => unimplemented!(),
                    _ => unimplemented!(),
                }
        }
    }
}

struct Monitor(Vec<KafkaActivity>);
impl Monitor {
    pub fn is_system_dead_enough(&mut self) -> bool {
        self.prune();

        unimplemented!()
    }

    fn prune(&mut self) {}
}
