#[macro_use]
extern crate lazy_static;
pub extern crate tokio;

pub mod env;
mod err;
pub mod katago;
pub mod websocket;

pub use bot_model::api::*;
pub use bot_model::*;
