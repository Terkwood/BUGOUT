extern crate bincode;
extern crate micro_model_moves;
extern crate tinybrain;
extern crate tungstenite;
extern crate uuid;

use micro_model_moves::*;
use tinybrain::{ComputeMove, MoveComputed};
use tungstenite::{connect, Message};
use uuid::Uuid;

fn wain() {
    let (mut socket, response) = connect("ws://localhost:3012/socket").expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
}

use std::net::TcpListener;
use std::thread::spawn;

use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};

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
                if msg.is_binary() || msg.is_text() {
                    websocket.write_message(msg).unwrap();
                }
            }
        });
    }
}
