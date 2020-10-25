use crate::env;
use bot_model::api::{ComputeMove, MoveComputed};

use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use futures_util::future::{select, Either};
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use std::time::Duration;
use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_hdr_async, tungstenite::Error};
use tungstenite::handshake::server::{Request, Response};
use tungstenite::http;
use tungstenite::{Message, Result};

pub async fn listen(opts: WSOpts) {
    let mut server = TcpListener::bind(&*env::ADDRESS).await.expect("WS bind");
    info!("WS bound to {}", &*env::ADDRESS);

    while let Ok((stream, _)) = server.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected stream should have a peer address");
        info!("Peer address: {}", peer);
        tokio::spawn(accept_connection(stream, opts.clone()));
    }
}

async fn accept_connection(stream: TcpStream, opts: WSOpts) {
    if let Err(e) = handle_connection(stream, &opts).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

const WRITE_TICK_MS: u64 = 10;

async fn handle_connection(stream: TcpStream, opts: &WSOpts) -> Result<()> {
    let callback = |req: &Request, response: Response| {
        if let Some(user_colon_pass) = &*env::AUTHORIZATION {
            let mut is_authorized = false;
            for (ref header, value) in req.headers() {
                if **header == "Authorization" {
                    // see https://en.wikipedia.org/wiki/Basic_access_authentication
                    is_authorized = *value == format!("Basic {}", base64::encode(user_colon_pass))
                }
            }
            if is_authorized {
                info!("Connection open");
                Ok(response)
            } else {
                warn!("No Auth");
                Err(http::response::Builder::new()
                    .status(401)
                    .body(None)
                    .expect("cannot form response"))
            }
        } else {
            info!("Connection open");
            Ok(response)
        }
    };
    let ws_stream = accept_hdr_async(stream, callback)
        .await
        .expect("failed to accept");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(WRITE_TICK_MS));
    let mut msg_fut = ws_receiver.next();
    let mut tick_fut = interval.next();
    loop {
        match select(msg_fut, tick_fut).await {
            Either::Left((msg, tick_fut_continue)) => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        match msg {
                            Message::Binary(data) => {
                                let move_computed: MoveComputed =
                                    deserialize(&data).expect("bincode deser");
                                if let Err(e) = opts.move_computed_in.send(move_computed) {
                                    error!("mc send err {:?}", e)
                                }
                            }
                            Message::Text(_) => warn!("Unexpected text data"),
                            Message::Close(_) => break,
                            Message::Ping(_) => (),
                            Message::Pong(_) => (),
                        }
                        tick_fut = tick_fut_continue;
                        msg_fut = ws_receiver.next();
                    }
                    None => break, // websocket stream terminated
                };
            }
            Either::Right((_, msg_fut_continue)) => {
                while let Ok(cm) = opts.compute_move_out.try_recv() {
                    ws_sender
                        .send(Message::Binary(serialize(&cm).expect("bincode ser")))
                        .await?;
                }

                msg_fut = msg_fut_continue;
                tick_fut = interval.next();
            }
        }
    }

    Ok(())
}

#[derive(Clone)]
pub struct WSOpts {
    pub compute_move_out: Receiver<ComputeMove>,
    pub move_computed_in: Sender<MoveComputed>,
}
impl WSOpts {
    pub fn from(c: &crate::registry::Components) -> Self {
        WSOpts {
            compute_move_out: c.compute_move_out.clone(),
            move_computed_in: c.move_computed_in.clone(),
        }
    }
}
