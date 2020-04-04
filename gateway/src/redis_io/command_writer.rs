use crate::backend_commands::BackendCommands;
use crate::redis_io::RedisPool;
use crossbeam_channel::{select, Receiver};

pub fn process_xadds(commands_in: Receiver<BackendCommands>, pool: &RedisPool) {
    loop {
        select! { recv(commands_in) -> _ => todo!() }
    }
}
