extern crate bincode;
extern crate community_redis_streams;
extern crate core_model;
extern crate crossbeam_channel;
extern crate log;
extern crate redis;

pub mod components;
mod game_lobby;
mod repo;
pub mod stream;
pub mod topics;

pub const PUBLIC_GAME_BOARD_SIZE: u16 = 19;
