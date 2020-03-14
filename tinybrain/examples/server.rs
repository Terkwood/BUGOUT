extern crate bincode;
extern crate micro_model_moves;
extern crate tinybrain;
extern crate tungstenite;
extern crate uuid;

use micro_model_moves::*;

use tinybrain::{ComputeMove, MoveComputed};
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::{connect, Message};
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
                            game_id: GameId(Uuid::nil()),
                            game_state: GameState {
                                board: Board::default(),
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
                let msg = websocket.read_message().unwrap();
                println!("Got msg {}", msg)
            }
        });
    }
}
