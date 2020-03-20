use crate::env;
use crossbeam_channel::{Receiver, Sender};
use log::warn;
use micro_model_bot::{ComputeMove, MoveComputed};
use std::net::TcpListener;
use std::thread;
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::http;
use tungstenite::Message;
use uuid::Uuid;

pub fn listen(opts: WSOpts) {
    let server = TcpListener::bind(&*env::ADDRESS).expect("WS bind");

    for stream in server.incoming() {
        let _move_computed_in = opts.move_computed_in.clone();
        let _compute_move_out = opts.compute_move_out.clone();
        thread::spawn(move || {
            let callback = |req: &Request, response: Response| {
                if let Some(user_colon_pass) = &*env::AUTHORIZATION {
                    let mut is_authorized = false;
                    for (ref header, value) in req.headers() {
                        if **header == "Authorization" {
                            is_authorized = *value == base64::encode(user_colon_pass)
                        }
                    }
                    if is_authorized {
                        Ok(response)
                    } else {
                        warn!("No Auth");
                        Err(http::response::Response::new(None))
                    }
                } else {
                    Ok(response)
                }
            };
            let mut _websocket = accept_hdr(stream.expect("stream"), callback);
            todo!("The Rest")
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
