use super::*;
use crate::backend_commands::SessionCommands;
pub fn choose(command: &SessionCommands) -> Backend {
    match command {
        &SessionCommands::Start {
            session_id: _,
            backend,
        } => backend,
        _ => Backend::Kafka,
    }
}
