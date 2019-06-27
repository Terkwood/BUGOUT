/// Adapted from https://raw.githubusercontent.com/housleyjk/ws-rs/master/examples/client.rs
extern crate env_logger;

/// Simple WebSocket client with error handling. It is not necessary to setup logging, but doing
/// so will allow you to see more details about the connection by using the RUST_LOG env variable.
extern crate ws;
use ws::connect;

fn main() {
    // Setup logging
    env_logger::init();

    // Connect to the url and call the closure
    if let Err(error) = connect("ws://127.0.0.1:3012", |out| {
        // Queue a message to be sent when the WebSocket is open
        if out
            .send(
                r#"{
                "type": "MakeMove",
                "gameId": "5cbfddce-101f-4415-a0d4-0e44d0403ce8",
                "requestId": "7dc12740-b7aa-492b-b3ce-0caeae253b92",
                "coord": {"x": 0, "y": 0},
                "player": "BLACK"
            }"#,
            )
            .is_err()
        {
            println!("Websocket couldn't queue an initial message.")
        } else {
            println!("Client sent message")
        }

        // The handler needs to take ownership of out, so we use move
        move |msg| {
            // Handle messages received on this connection
            println!("Client got message '{}'. ", msg);
            Ok(())
        }
    }) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to: {:?}", error);
    }
}
