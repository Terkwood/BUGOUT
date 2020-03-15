use crate::*;
use crossbeam_channel::{select, Receiver, Sender};
use http::Request;
use tungstenite::util::NonBlockingResult;
use tungstenite::{connect, Message};

mod authorization;

pub fn start(compute_move_in: Sender<ComputeMove>, move_computed_out: Receiver<MoveComputed>) {
    let (mut socket, response) =
        connect(create_request()).expect("cannot connect to robocall host");
    println!("Connected to robocall, http status: {}", response.status());

    println!("Headers follow:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header)
    }

    loop {
        let incoming_data = socket.read_message().no_block();
        match incoming_data {
            Err(e) => println!("Error reading incoming data {:?}", e),
            Ok(None) => println!("Nothing on the line"), // TODO
            Ok(Some(tungstenite::Message::Binary(data))) => {
                let cm: Result<ComputeMove, _> = bincode::deserialize(&data);
                match cm {
                    Err(e) => println!("failed to deser compute move {:?}", e),
                    Ok(compute_move) => {
                        if let Err(e) = compute_move_in.send(compute_move) {
                            println!("failed to send compute move {:?}", e)
                        }
                    }
                }
            }
            Ok(_e) => println!("requires binary"),
        };
        select! {
            recv(move_computed_out) -> mc =>
                match mc {
                    Err(e) => println!("Error reading move_computed_out {:?}",e),
                    Ok(move_computed) => {
                        socket
                            .write_message(
                                Message::Binary(
                                    bincode::serialize(&move_computed)
                                        .expect("bincode move computed")
                                    )
                                ).expect("write websocket message");
                            println!("Wrote on socket")
                        }
            }
        }
    }
}

fn create_request() -> http::Request<()> {
    let mut request = Request::builder().uri(&*env::ROBOCALL_URL);

    if let Some(h) = authorization::header() {
        request = request.header("Authorization", h);
    }

    request.body(()).expect("request")
}
