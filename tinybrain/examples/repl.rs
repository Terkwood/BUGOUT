/// Write moves to stdin as BLACK and let the tinybrain respond
///
/// This example starts a barebones WS server and pushes
/// your moves to tinybrain
extern crate bincode;
extern crate micro_model_moves;
extern crate tinybrain;
extern crate tungstenite;
extern crate uuid;

use micro_model_moves::*;

use log::info;
use std::net::TcpListener;
use std::thread::spawn;
use tinybrain::{ComputeMove, MoveComputed};
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::Message;
use uuid::Uuid;

fn main() {
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |_: &Request, response: Response| Ok(response);
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            let mut game_state = GameState {
                board: Board {
                    size: 9,
                    ..Board::default()
                },
                captures: Captures::default(),
                moves: vec![],
                player_up: Player::BLACK,
                turn: 1,
            };
            let game_id = GameId(Uuid::new_v4());

            loop {
                todo!("read from stdin");
                todo!("convert i guess");
                websocket
                    .write_message(Message::Binary(
                        bincode::serialize(&ComputeMove {
                            game_id,
                            game_state,
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
                    _ => info!("Got another response"),
                }
            }
        });
    }
}
