use crate::*;

use crossbeam_channel::{select, Receiver, Sender};
use http::Request;
use log::{error, info, trace, warn};
use std::net::SocketAddr;
use std::time::Duration;
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
mod authorization;

pub async fn start(
    compute_move_in: Sender<ComputeMove>,
    move_computed_out: Receiver<MoveComputed>,
) {
    let (mut socket, response) = connect_async(create_request())
        .await
        .expect("cannot connect to botlink host");
    trace!("Connected to botlink, http status: {}", response.status());

    let (mut write, mut read) = socket.split();

    let mut interval = tokio::time::interval(Duration::from_millis(WRITE_TICK_MS));
    let mut msg_fut = read.next();
    let mut tick_fut = interval.next();
    todo!("hack loop");
    /*
    loop {
        if let Ok(incoming_data) = todo!() {
            match incoming_data {
                tungstenite::Message::Binary(data) => {
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
                _ => warn!("unmatched message"),
            }
        };
        select! {
            recv(move_computed_out) -> mc =>
                match mc {
                    Err(e) => error!("Error reading move_computed_out {:?}",e),
                    Ok(move_computed) => {
                        todo!();
                        if let Ok(m) = socket.next().await.expect("socket"){
                        m.write();
                        }
                        /*let wr_m = socket.write_message(
                                Message::Binary(
                                    bincode::serialize(&move_computed)
                                        .expect("bincode move computed")
                                    )
                                ).expect("write websocket message");*/
                            trace!("Wrote on socket")
                        }
            }
        }
    }*/
}

const WRITE_TICK_MS: u64 = 10;

fn create_request() -> http::Request<()> {
    let mut request = Request::builder().uri(&*env::BOTLINK_URL);

    if let Some(h) = authorization::header() {
        request = request.header("Authorization", h);
    }

    request.body(()).expect("request")
}
