mod entry_id_repo;
mod game_lobby_repo;

pub use entry_id_repo::*;
pub use game_lobby_repo::*;
use redis::Client;
use std::sync::Arc;

#[derive(Debug)]
pub enum FetchErr {
    EIDRepo,
}

#[derive(Debug)]
pub enum WriteErr {
    EIDRepo,
}

pub struct RedisRepo {
    pub client: Arc<Client>,
    pub key_provider: KeyProvider,
}

const DEFAULT_NAMESPACE: &str = "BUGOUT";
#[derive(Clone, Debug)]
pub struct RedisKeyNamespace(pub String);
impl Default for RedisKeyNamespace {
    fn default() -> Self {
        RedisKeyNamespace(DEFAULT_NAMESPACE.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct KeyProvider(pub RedisKeyNamespace);
impl Default for KeyProvider {
    fn default() -> Self {
        KeyProvider(RedisKeyNamespace::default())
    }
}
impl KeyProvider {
    pub fn entry_ids(&self) -> String {
        format!("/{}/micro_game_lobby/entry_ids", (self.0).0)
    }
    pub fn game_lobby(&self) -> String {
        format!("/{}/micro_game_lobby/game_lobby", (self.0).0)
    }
}
