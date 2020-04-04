use crate::backend::repo::SessionBackendRepo;
use crate::backend_commands::BackendCommands;
use crossbeam_channel::{select, Receiver};

pub fn process_xadds(
    commands_in: Receiver<BackendCommands>,
    _sb_repo: Box<dyn SessionBackendRepo>,
) {
    loop {
        select! { recv(commands_in) -> _ => todo!() }
    }
}
