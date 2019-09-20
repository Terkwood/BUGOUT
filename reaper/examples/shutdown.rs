extern crate reaper;

use crossbeam_channel::{unbounded, Receiver, Sender};

use reaper::model::ShutdownCommand;
use reaper::*;

/// Shut down the instance based on the .env
/// configuration for its name
pub fn main() {
    let (shutdown_in, shutdown_out): (Sender<ShutdownCommand>, Receiver<ShutdownCommand>) =
        unbounded();

    env::init();

    shutdown_in.send(ShutdownCommand::new()).unwrap();

    shutdown::listen(shutdown_out);
}
