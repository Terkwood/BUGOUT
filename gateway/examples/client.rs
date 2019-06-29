/// Adapted from https://raw.githubusercontent.com/housleyjk/ws-rs/master/examples/client.rs
extern crate env_logger;

extern crate uuid;
/// Simple WebSocket client with error handling. It is not necessary to setup logging, but doing
/// so will allow you to see more details about the connection by using the RUST_LOG env variable.
extern crate ws;

use uuid::Uuid;
use ws::{connect, CloseCode};

fn main() {
    // Setup logging
    env_logger::init();

    // Connect to the url and call the closure
    if let Err(error) = connect("ws://127.0.0.1:3012", |out| {
        // Queue a message to be sent when the WebSocket is open

        let game_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let msg = format!("{{\"type\":\"MakeMove\",\"gameId\":\"{:?}\",\"reqId\":\"{:?}\",\"player\":\"BLACK\",\"coord\":{{\"x\":0,\"y\":0}}}}", game_id, request_id).to_string();

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
