use crate::backend_commands::BackendCommands;

use crossbeam_channel::{select, Receiver, Sender};
use log::error;
pub fn double_commands(opts: DoublerOpts) {
    loop {
        select! {
            recv(opts.session_commands_out) -> msg => match msg {
                Ok(  backend_command ) => {
                    if let Err(e) = opts.redis_commands_in.send(backend_command.clone()) {
                        error!("err doubler 0 {:?}",e)
                    }

                    if let Err(e) = opts.kafka_commands_in.send(backend_command) {
                        error!("FAILED doubler TO BACKEND {:?}", e)
                    }
                }
                Err(e) => error!("session command out: {:?}",e)
            }
        }
    }
}

pub struct DoublerOpts {
    pub session_commands_out: Receiver<BackendCommands>,
    pub kafka_commands_in: Sender<BackendCommands>,
    pub redis_commands_in: Sender<BackendCommands>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend_commands::*;
    use crate::model::*;

    use crossbeam_channel::{select, unbounded};
    use std::thread;
    use uuid::Uuid;

    #[test]
    fn test_double_commands() {
        let (session_commands_in, session_commands_out): (
            Sender<BackendCommands>,
            Receiver<BackendCommands>,
        ) = unbounded();

        let (kafka_commands_in, kafka_commands_out): (
            Sender<BackendCommands>,
            Receiver<BackendCommands>,
        ) = unbounded();

        let (redis_commands_in, redis_commands_out): (
            Sender<BackendCommands>,
            Receiver<BackendCommands>,
        ) = unbounded();
        thread::spawn(move || {
            let opts = DoublerOpts {
                kafka_commands_in,
                redis_commands_in,
                session_commands_out,
            };
            double_commands(opts)
        });

        {
            let session_id = Uuid::new_v4();
            let client_id = Uuid::new_v4();
            session_commands_in
                .send(BackendCommands::FindPublicGame(
                    FindPublicGameBackendCommand {
                        session_id,
                        client_id,
                    },
                ))
                .expect("send0")
        }

        {
            let game_id = micro_model_moves::GameId(Uuid::new_v4());
            let player = Player::WHITE;
            let board_size = Some(9 as u8);
            session_commands_in
                .send(BackendCommands::AttachBot(
                    micro_model_bot::gateway::AttachBot {
                        game_id,
                        player: match player {
                            Player::WHITE => micro_model_moves::Player::WHITE,
                            _ => micro_model_moves::Player::BLACK,
                        },
                        board_size,
                    },
                ))
                .expect("send1")
        }

        select! { recv(kafka_commands_out) -> co =>
            match co.expect("kafka co 0 select") {
                BackendCommands::FindPublicGame(_) => assert!(true),
                _ => assert!(false)
            }
        }

        select! { recv(redis_commands_out) -> co =>
            match co.expect("redis co 0 select") {
                BackendCommands::FindPublicGame(_) => assert!(true),
                _ => assert!(false)
            }
        }

        select! { recv(kafka_commands_out) -> co =>
            match co.expect("kafka co 1 select") {
                BackendCommands::AttachBot(_) => assert!(true),
                _ => assert!(false)
            }
        }

        select! { recv(redis_commands_out) -> co =>
            match co.expect("redis co 1 select") {
                BackendCommands::AttachBot(_) => assert!(true),
                _ => assert!(false)
            }
        }
    }
}
