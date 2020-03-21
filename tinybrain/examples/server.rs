extern crate bincode;
extern crate micro_model_bot;
extern crate micro_model_moves;
extern crate tinybrain;
extern crate tungstenite;
extern crate uuid;

use log::{error, info};
use micro_model_bot::*;
use micro_model_moves::*;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::Message;
use uuid::Uuid;

fn main() {
    env_logger::init();
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |req: &Request, response: Response| {
                info!("Received a new ws handshake");
                info!("The request's path is: {}", req.uri().path());
                info!("The request's headers are:");
                for (ref header, _value) in req.headers() {
                    info!("* {}", header);
                }

                Ok(response)
            };
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            loop {
                websocket
                    .write_message(Message::Binary(
                        bincode::serialize(&ComputeMove {
                            game_id: GameId(Uuid::new_v4()),
                            game_state: GameState {
                                board: Board {
                                    size: 9,
                                    ..Board::default()
                                },
                                captures: Captures::default(),
                                moves: vec![],
                                player_up: Player::BLACK,
                                turn: 1,
                            },
                        })
                        .expect("ser"),
                    ))
                    .unwrap();

                // block
                match websocket.read_message().unwrap() {
                    Message::Binary(data) => {
                        let move_computed: MoveComputed =
                            bincode::deserialize(&data).expect("bincode deser");
                        info!("Got move computed {:?}", move_computed);
                    }
                    _ => error!("Got another response"),
                }
            }
        });
    }
}
