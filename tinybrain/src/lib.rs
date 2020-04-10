#[macro_use]
extern crate futures_util;
#[macro_use]
extern crate lazy_static;
extern crate base64;
extern crate bincode;
extern crate dotenv;
extern crate env_logger;
extern crate http;
extern crate log;
extern crate micro_model_moves;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
pub extern crate tokio;
extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate uuid;

pub mod env;
mod err;
pub mod katago;
pub mod websocket;

pub use micro_model_bot::*;
