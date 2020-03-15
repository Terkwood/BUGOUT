extern crate base64;
extern crate bincode;
extern crate dotenv;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate tungstenite;
extern crate uuid;
#[macro_use]
extern crate lazy_static;
extern crate http;

pub mod env;
mod err;
pub mod katago;
pub mod websocket;

use micro_model_bot::*;
