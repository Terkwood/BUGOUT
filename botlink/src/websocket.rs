use crate::env;
use crossbeam_channel::{Receiver, Sender};
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
            let callback = |req: &Request, _response: Response| {
                println!("Received a new ws handshake");
                println!("The request's path is: {}", req.uri().path());
                println!("The request's headers are:");
                for (ref header, _value) in req.headers() {
                    println!("* {}", header);
                    todo!("Auth")
                }

                todo!("Auth")
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
