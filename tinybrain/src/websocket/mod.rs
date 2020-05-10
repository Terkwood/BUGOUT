mod authorization;

use crate::*;

use bincode;
use crossbeam_channel::{Receiver, Sender};
use future::{select, Either};
use futures_util::stream::SplitSink;
use futures_util::{future, SinkExt, StreamExt};
use http::Request;
use log::{error, info, warn};
use std::cmp;
use std::thread;
use std::time::Duration;
use tokio_tls::TlsStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const RETRY_MAX_EXP: u32 = 5;
const RETRY_BASE_SECS: u64 = 2;

pub async fn start(
    compute_move_in: Sender<ComputeMove>,
    move_computed_out: Receiver<MoveComputed>,
) {
    let mut retry_exp: u32 = 0;
    loop {
        retry_exp = match connect_loop(compute_move_in.clone(), move_computed_out.clone()).await {
            InitialConnection::Succeeded => 0,
            InitialConnection::Failed => cmp::min(retry_exp + 1, RETRY_MAX_EXP),
        };

        let sleep_secs = RETRY_BASE_SECS.pow(retry_exp);
        warn!(
            "Connection failed. Retrying botlink ({}) in {} seconds...",
            &*env::BOTLINK_URL,
            sleep_secs
        );
        thread::sleep(Duration::from_secs(sleep_secs))
    }
}

const WRITE_TICK_MS: u64 = 10;

/// Were we able to establish a connection to botlink
/// at any point?  If so, this enum indicates success.
#[derive(Copy, Debug, Clone)]
enum InitialConnection {
    Succeeded,
    Failed,
}

/// Connect to botlink service and loop forever, handling
/// ws requests from botlink and sending responses as katago
/// computes moves.
/// Returns a value based on whether there was any initial
/// success with a connection to botlink.  Connecting, processing
/// some moves, and then being drops counts as a successful
/// initial connection.
async fn connect_loop(
    compute_move_in: Sender<ComputeMove>,
    move_computed_out: Receiver<MoveComputed>,
) -> InitialConnection {
    if let Ok((socket, response)) = connect_async(create_http_request()).await {
        info!(
            "Connected to botlink ({}), http status: {}",
            &*env::BOTLINK_URL,
            response.status()
        );

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
                                    handle_compute_move(data, &compute_move_in)
                                }
                                Message::Text(_) => warn!("Unexpected text data"),
                                Message::Close(_) => break InitialConnection::Succeeded,
                                _ => (), // PingPong
                            }
                        }

                        write_tick_fut = write_tick_fut_continue;
                        read_msg_fut = read.next();
                    }
                    None => break InitialConnection::Succeeded, // ws stream terminated
                },
                Either::Right((_, read_msg_fut_continue)) => {
                    respond_all_moves_computed(&mut write, &move_computed_out).await;

                    read_msg_fut = read_msg_fut_continue;
                    write_tick_fut = interval.next();
                }
            }
        }
    } else {
        InitialConnection::Failed
    }
}

/// Iterate through all outstanding move_computed results
/// that katago has delivered through crossbeam channel,
/// and send each one to botlink via websocket.
async fn respond_all_moves_computed(
    write: &mut SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::stream::Stream<
                tokio::net::TcpStream,
                TlsStream<tokio::net::TcpStream>,
            >,
        >,
        tokio_tungstenite::tungstenite::Message,
    >,
    move_computed_out: &Receiver<MoveComputed>,
) {
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
}

/// Deserialize the compute move request received from botlink,
/// then send it over crossbeam to be handled by the katago thread.
fn handle_compute_move(data: Vec<u8>, compute_move_in: &Sender<ComputeMove>) {
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

/// Creates an HTTP request, which will be used to connect
/// to botlink service
fn create_http_request() -> http::Request<()> {
    let mut request = Request::builder().uri(&*env::BOTLINK_URL);

    if let Some(h) = authorization::header() {
        request = request.header("Authorization", h);
    }

    request.body(()).expect("request")
}
