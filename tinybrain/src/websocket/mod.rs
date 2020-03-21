use crate::*;
use crossbeam_channel::{select, Receiver, Sender};
use http::Request;
use log::{error, trace, warn};
use tungstenite::util::NonBlockingResult;
use tungstenite::{connect, Message};
mod authorization;

pub fn start(compute_move_in: Sender<ComputeMove>, move_computed_out: Receiver<MoveComputed>) {
    let (mut socket, response) =
        connect(create_request()).expect("cannot connect to robocall host");
    trace!("Connected to botlink, http status: {}", response.status());

    trace!("Headers follow:");
    for (ref header, _value) in response.headers() {
        trace!("* {}", header)
    }

    loop {
        let incoming_data = socket.read_message().no_block();
        match incoming_data {
            Err(e) => error!("Error reading incoming data {:?}", e),
            Ok(None) => trace!("Empty read in ws"),
            Ok(Some(tungstenite::Message::Binary(data))) => {
                let cm: Result<ComputeMove, _> = bincode::deserialize(&data);
                match cm {
                    Err(e) => error!("failed to deser compute move {:?}", e),
                    Ok(compute_move) => {
                        if let Err(e) = compute_move_in.send(compute_move) {
                            error!("failed to send compute move {:?}", e)
                        }
                    }
                }
            }
            Ok(_e) => warn!("requires binary"),
        };
        select! {
            recv(move_computed_out) -> mc =>
                match mc {
                    Err(e) => error!("Error reading move_computed_out {:?}",e),
                    Ok(move_computed) => {
                        socket
                            .write_message(
                                Message::Binary(
                                    bincode::serialize(&move_computed)
                                        .expect("bincode move computed")
                                    )
                                ).expect("write websocket message");
                            error!("Wrote on socket")
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
