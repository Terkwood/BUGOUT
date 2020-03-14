extern crate bincode;
extern crate micro_model_moves;
extern crate tinybrain;
extern crate tungstenite;
extern crate uuid;

use micro_model_moves::*;
use tinybrain::{ComputeMove, MoveComputed};
use tungstenite::{connect, Message};
use uuid::Uuid;

fn main() {
    let (mut socket, response) = connect("ws://localhost:3012/socket").expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    socket
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
    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
}
