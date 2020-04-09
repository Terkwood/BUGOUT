use crate::env;
use bincode::{deserialize, serialize};
use crossbeam_channel::{select, Receiver, Sender};
use log::{info, error, warn};
use micro_model_bot::{ComputeMove, MoveComputed};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::http;
use tungstenite::util::NonBlockingResult;
use tungstenite::Message;

pub fn listen(opts: WSOpts) {
    let server = TcpListener::bind(&*env::ADDRESS).expect("WS bind");
    info!("WS bound to {}", &*env::ADDRESS);

    for stream in server.incoming() {
        let move_computed_in = opts.move_computed_in.clone();
        let compute_move_out = opts.compute_move_out.clone();
        thread::spawn(move || {
            let callback = |req: &Request, response: Response| {
                if let Some(user_colon_pass) = &*env::AUTHORIZATION {
                    let mut is_authorized = false;
                    for (ref header, value) in req.headers() {
                        if **header == "Authorization" {
                            // see https://en.wikipedia.org/wiki/Basic_access_authentication
                            is_authorized =
                                *value == format!("Basic {}", base64::encode(user_colon_pass))
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
            let mut websocket = accept_hdr(stream.expect("stream"), callback).expect("websocket");
            loop {
                select! {
                    recv(compute_move_out) -> msg => match msg {
                        Ok(cm) => {
                            let msg = Message::Binary(serialize(&cm).expect("bincode ser"));
                            websocket
                                .write_message(msg)
                                .expect("ws write")
                        },
                        Err(e) => error!("error receiving cm {:?}",e)
                    },
                    default(Duration::from_millis(10)) => {
                        while let Some(Message::Binary(data)) = websocket.read_message().no_block().expect("read no block")  {
                            let move_computed: MoveComputed =
                                deserialize(&data).expect("bincode deser");
                            if let Err(e) = move_computed_in.send(move_computed) {
                                error!("mc send err {:?}",e)
                            }
                        }
                    },
                }
            }
        });
    }
}

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
