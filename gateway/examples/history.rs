extern crate rand;
extern crate uuid;
extern crate ws;

use uuid::Uuid;
use ws::connect;

fn main() {
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

    // stop the example at some point in the future
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        std::process::exit(0);
    });

    // Connect to the url and call the closure
    if let Err(error) = connect("ws://127.0.0.1:3012", |out| {
        // Queue a message to be sent when the WebSocket is open

        let game_id: Uuid = game_id_parsed.clone().unwrap();
        let request_id = Uuid::new_v4();

        // We need to "reconnect" to a game in progress, or we won't receive
        // any reply once the history is provided
        let reconn_msg = format!(
            "{{\"type\":\"Reconnect\",\"gameId\":\"{:?}\",\"reqId\":\"{:?}\"}}",
            game_id, request_id
        )
        .to_string();
        out.send(reconn_msg).expect("sent");

        let msg = format!(
            "{{\"type\":\"ProvideHistory\",\"gameId\":\"{:?}\",\"reqId\":\"{:?}\"}}",
            game_id, request_id
        )
        .to_string();
        let debug_msg = String::from(&msg);

        if out.send(msg).is_err() {
            println!("Websocket couldn't queue an initial message.")
        } else {
            println!("Client sent message {}", debug_msg)
        }

        |recv_msg| {
            // Handle messages received on this connection
            println!("Client got message {} ", recv_msg);

            Ok(())
        }
    }) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to: {:?}", error);
    }
}
