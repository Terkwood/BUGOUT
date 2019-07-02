/// Adapted from https://raw.githubusercontent.com/housleyjk/ws-rs/master/examples/client.rs
extern crate uuid;
/// Simple WebSocket client with error handling. It is not necessary to setup logging, but doing
/// so will allow you to see more details about the connection by using the RUST_LOG env variable.
extern crate ws;

use uuid::Uuid;
use ws::{connect, CloseCode};

fn main() {
    // Connect to the url and call the closure
    if let Err(error) = connect("ws://127.0.0.1:3012", |out| {
        // Queue a message to be sent when the WebSocket is open

        // This game_id needs to match one that is currently available in the system,
        // or judge will crash.
        let game_id: Uuid = Uuid::parse_str("7335be5f-468d-4bf1-9d33-1c582b26ec5a").unwrap();

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
