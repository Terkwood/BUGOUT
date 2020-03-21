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

use log::{error, info, trace};
use std::net::TcpListener;
use std::thread::spawn;
use text_io::read;
use tinybrain::katago::json;
use tinybrain::{ComputeMove, MoveComputed};
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::Message;
use uuid::Uuid;

const PLAYER: Player = Player::BLACK;
const OPPONENT: Player = Player::WHITE;

fn main() {
    env_logger::init();
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |_: &Request, response: Response| {
                info!("# Received WS handshake");
                Ok(response)
            };
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
                info!("< B");
                let line: String = read!("{}\n");
                let happy_coord = json::interpret_coord(&line);
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
                        game_state.player_up = OPPONENT;
                        if let Some(c) = coord {
                            game_state.board.pieces.insert(c, PLAYER);
                        }

                        trace!("(writing to websocket)");
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
                                let mmm = json::Move::from(last_move.player, last_move.coord)
                                    .expect("boom");
                                info!("> {} {}", mmm.0, (mmm.1).0);

                                game_state.moves.push(MoveMade {
                                    coord: last_move.coord,
                                    event_id: EventId::new(),
                                    game_id: game_id.clone(),
                                    player: OPPONENT,
                                    captured: vec![], // Ignored by katago
                                    reply_to: ReqId(Uuid::nil()),
                                });
                                game_state.player_up = PLAYER;
                                game_state.turn += 1;
                                if let Some(c) = last_move.coord {
                                    game_state.board.pieces.insert(c, OPPONENT);
                                }
                            }
                            _ => error!(">>> FAIL <<<"),
                        }
                    }
                }
            }
        });
    }
}
