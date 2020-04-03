use crate::backend::repo::SessionBackendRepo;
use crate::backend_commands::BackendCommands;
use crossbeam_channel::{select, unbounded, Receiver, Sender};

pub fn start(commands_in: Receiver<BackendCommands>, sb_repo: Box<dyn SessionBackendRepo>) {
    loop {
        select! { recv(commands_in) -> _ => todo!() }
    }
}
