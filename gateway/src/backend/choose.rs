use super::*;
use crate::backend_commands::SessionCommands;

/// For now, a trivial choice
pub fn fallback(command: &SessionCommands) -> Backend {
    match command {
        &SessionCommands::StartBotSession {
            session_id: _,
            bot_player: _,
            board_size: _,
        } => Backend::RedisStreams,
        _ => Backend::Kafka,
    }
}
