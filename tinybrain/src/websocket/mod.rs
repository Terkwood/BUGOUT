mod authorization;

use crate::*;

use bincode;
use crossbeam_channel::{Receiver, Sender};
use future::{select, Either};
use futures_util::{future, SinkExt, StreamExt};
use http::Request;
use log::{error, info, warn};
use std::cmp;
use std::thread;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const RETRY_MAX_EXP: u32 = 5;
const RETRY_BASE_SECS: u64 = 2;
pub async fn start(
    compute_move_in: Sender<ComputeMove>,
    move_computed_out: Receiver<MoveComputed>,
) {
    let mut retry_exp: u32 = 0;
    loop {
        retry_exp = match process_loop(compute_move_in.clone(), move_computed_out.clone()).await {
            ConnectionResult::Succeeded => 0,
            ConnectionResult::Failed => cmp::min(retry_exp + 1, RETRY_MAX_EXP),
        };

        let sleep_secs = RETRY_BASE_SECS.pow(retry_exp);
        info!("Retrying botlink in {} seconds...", sleep_secs);
        thread::sleep(Duration::from_secs(sleep_secs))
    }
}

const WRITE_TICK_MS: u64 = 10;
#[derive(Copy, Debug, Clone)]
enum ConnectionResult {
    Succeeded,
    Failed,
}
async fn process_loop(
    compute_move_in: Sender<ComputeMove>,
    move_computed_out: Receiver<MoveComputed>,
) -> ConnectionResult {
    if let Ok((socket, response)) = connect_async(create_request()).await {
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
                                Message::Close(_) => break ConnectionResult::Succeeded,
                                _ => (), // PingPong
                            }
                        }

                        write_tick_fut = write_tick_fut_continue;
                        read_msg_fut = read.next();
                    }
                    None => break ConnectionResult::Succeeded, // ws stream terminated
                },
                Either::Right((_, read_msg_fut_continue)) => {
                    while let Ok(mc) = move_computed_out.try_recv() {
                        if let Err(e) = write
                            .send(Message::Binary(bincode::serialize(&mc).expect("ser")))
                            .await
                        {
                            error!("write {}", e)
                        } else {
                            info!("🆗 {:?}", mc)
                        }
                    }

                    read_msg_fut = read_msg_fut_continue;
                    write_tick_fut = interval.next();
                }
            }
        }
    } else {
        ConnectionResult::Failed
    }
}

fn create_request() -> http::Request<()> {
    let mut request = Request::builder().uri(&*env::BOTLINK_URL);

    if let Some(h) = authorization::header() {
        request = request.header("Authorization", h);
    }

    request.body(()).expect("request")
}
