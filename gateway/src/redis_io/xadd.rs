use crate::backend_commands::BackendCommands;
use crate::redis_io::RedisPool;

use crossbeam_channel::{select, Receiver};
use log::{error, warn};

pub fn xadd_commands(commands_in: Receiver<BackendCommands>, _pool: &RedisPool) {
    loop {
        select! {
            recv(commands_in) -> backend_command_msg => match backend_command_msg {
                Err(e) => error!("backend command xadd {:?}",e),
                Ok(command) => xadd(command)
            }
        }
    }
}

fn xadd(command: BackendCommands) {
    match command {
        BackendCommands::AttachBot(ab_cmd) => {
            warn!("Failed to match attach bot command : {:?}", ab_cmd)
        }
        _ => error!("match"),
    }
}
