mod entry_id_repo;
mod game_lobby_repo;

pub use entry_id_repo::*;
pub use game_lobby_repo::*;

#[derive(Debug)]
pub enum FetchErr {}

#[derive(Debug)]
pub struct WriteErr;
