mod entry_id_repo;
mod game_lobby_repo;

pub use entry_id_repo::*;
pub use game_lobby_repo::*;

#[derive(Debug)]
pub enum FetchErr {
    EIDRepo,
}

#[derive(Debug)]
pub enum WriteErr {
    EIDRepo,
}

pub const ENTRY_ID_KEY: &str = "/BUGOUT/micro_game_lobby/entry_ids";
pub const _GAME_LOBBY_KEY: &str = "/BUGOUT/micro_game_lobby/game_lobby";
