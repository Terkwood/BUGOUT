/// Write moves to stdin as BLACK and let the tinybrain respond
///
/// This example starts a barebones WS server and pushes
/// your moves to tinybrain
extern crate bincode;
extern crate micro_model_moves;
extern crate text_io;
extern crate tinybrain;
extern crate tungstenite;
extern crate uuid;

use micro_model_moves::*;

use log::{error, info};
use std::net::TcpListener;
use std::thread::spawn;
use text_io::read;
use tinybrain::katago::json::interpret_coord;
use tinybrain::katago::json::Move;
use tinybrain::{ComputeMove, MoveComputed};
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::Message;
use uuid::Uuid;

const PLAYER: Player = Player::BLACK;

fn main() {
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |_: &Request, response: Response| Ok(response);
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            let size = 9;
            info!("# Board size {}x{}", size, size);

            let game_state = &mut GameState {
                board: Board {
                    size,
                    ..Board::default()
                },
                captures: Captures::default(),
                moves: vec![],
                player_up: Player::BLACK,
                turn: 1,
            };
            let game_id = &GameId(Uuid::new_v4());

            loop {
                let word: String = read!();
                let happy_coord = interpret_coord(&word);
                print!("< B ");
                match happy_coord {
                    Err(_) => {
                        error!("! parse error");
                        continue;
                    }
                    Ok(coord) => {
                        game_state.moves.push(MoveMade {
                            coord,
                            event_id: EventId::new(),
                            game_id: game_id.clone(),
                            player: PLAYER,
                            captured: vec![], // Ignored by katago
                            reply_to: ReqId(Uuid::nil()),
                        });
                        game_state.turn += 1;

                        websocket
                            .write_message(Message::Binary(
                                bincode::serialize(&ComputeMove {
                                    game_id: game_id.clone(),
                                    game_state: game_state.clone(),
                                })
                                .expect("ser"),
                            ))
                            .unwrap();

                        // block
                        match websocket.read_message().unwrap() {
                            Message::Binary(data) => {
                                let move_computed: MoveComputed =
                                    bincode::deserialize(&data).expect("bincode deser");
                                let last_move = move_computed.0;
                                info!(
                                    "> {}",
                                    Move::from(last_move.player, last_move.coord)
                                        .expect("boom")
                                        .0
                                );
                            }
                            _ => error!(">>> FAIL <<<"),
                        }
                    }
                }
            }
        });
    }
}
