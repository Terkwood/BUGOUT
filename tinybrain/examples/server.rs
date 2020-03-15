extern crate bincode;
extern crate micro_model_moves;
extern crate tinybrain;
extern crate tungstenite;
extern crate uuid;

use micro_model_moves::*;

use tinybrain::{ComputeMove, MoveComputed};
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::Message;
use uuid::Uuid;

use std::net::TcpListener;
use std::thread::spawn;

fn main() {
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |req: &Request, response: Response| {
                println!("Received a new ws handshake");
                println!("The request's path is: {}", req.uri().path());
                println!("The request's headers are:");
                for (ref header, _value) in req.headers() {
                    println!("* {}", header);
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
                        println!("Got move computed {:?}", move_computed);
                    }
                    _ => println!("Got another response"),
                }
            }
        });
    }
}
