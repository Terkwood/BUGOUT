extern crate rand;
extern crate uuid;
extern crate ws;

use rand::Rng;
use uuid::Uuid;
use ws::{connect, CloseCode};

fn main() {
    let mut rng = rand::thread_rng();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("You must specify a game ID as a command line argument")
    }

    let game_id_parsed = Uuid::parse_str(&args[1]);

    if let Err(parse_fail) = game_id_parsed {
        panic!(
            "Couldn't parse a UUID game ID on command line: {:?}",
            parse_fail
        )
    }

    // Connect to the url and call the closure
    if let Err(error) = connect("ws://127.0.0.1:3012", |out| {
        // Queue a message to be sent when the WebSocket is open

        let game_id: Uuid = game_id_parsed.unwrap();
        let request_id = Uuid::new_v4();

        let (x, y) = (rng.gen_range(0, 19), rng.gen_range(0, 19));
        let msg = format!("{{\"type\":\"ProvideHistory\",\"gameId\":\"{:?}\",\"reqId\":\"{:?}\"}}", game_id, request_id).to_string();

        if out.send(msg).is_err() {
            println!("Websocket couldn't queue an initial message.")
        } else {
            println!("Client sent message with req_id: {:?}", request_id)
        }

        // The handler needs to take ownership of out, so we use move
        move |msg| {
            // Handle messages received on this connection
            println!("Client got message '{}'. ", msg);
            out.close(CloseCode::Normal)
        }
    }) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to: {:?}", error);
    }
}
