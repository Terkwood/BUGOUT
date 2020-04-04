use super::*;
use crate::backend_commands::SessionCommands;

/// For now, a trivial choice
pub fn fallback(command: &SessionCommands) -> Backend {
    match command {
        &SessionCommands::Start {
            session_id: _,
            backend,
        } => backend,
        _ => Backend::Kafka,
    }
}
