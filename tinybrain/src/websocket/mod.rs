mod authorization;

use crate::*;

use bincode;
use crossbeam_channel::{Receiver, Sender};
use future::{select, Either};
use futures_util::{future, SinkExt, StreamExt};
use http::Request;
use log::{error, info, warn};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const WRITE_TICK_MS: u64 = 10;
pub async fn start(
    compute_move_in: Sender<ComputeMove>,
    move_computed_out: Receiver<MoveComputed>,
) {
    let (socket, response) = connect_async(create_request())
        .await
        .expect("cannot connect to botlink host");
    info!("Connected to botlink, http status: {}", response.status());

    let (mut write, mut read) = socket.split();

    let mut interval = tokio::time::interval(Duration::from_millis(WRITE_TICK_MS));
    let mut read_msg_fut = read.next();
    let mut write_tick_fut = interval.next();
    loop {
        match select(read_msg_fut, write_tick_fut).await {
            Either::Left((msg, write_tick_fut_continue)) => match msg {
                Some(msg) => {
                    if let Ok(msg) = msg {
                        match msg {
                            Message::Binary(data) => {
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
                            Message::Text(_) => warn!("Unexpected text data"),
                            Message::Close(_) => break,
                            _ => (), // PingPong
                        }
                    }

                    write_tick_fut = write_tick_fut_continue;
                    read_msg_fut = read.next();
                }
                None => break, // ws stream terminated
            },
            Either::Right((_, read_msg_fut_continue)) => {
                while let Ok(mc) = move_computed_out.try_recv() {
                    if let Err(e) = write
                        .send(Message::Binary(bincode::serialize(&mc).expect("ser")))
                        .await
                    {
                        error!("write {}", e)
                    }
                }

                read_msg_fut = read_msg_fut_continue;
                write_tick_fut = interval.next();
            }
        }
    }
}

fn create_request() -> http::Request<()> {
    let mut request = Request::builder().uri(&*env::BOTLINK_URL);

    if let Some(h) = authorization::header() {
        request = request.header("Authorization", h);
    }

    request.body(()).expect("request")
}
