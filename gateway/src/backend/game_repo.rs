use super::{Backend, BackendRepoErr};
use crate::model::GameId;

pub trait GameBackendRepo {
    fn get(&self, game_id: &GameId) -> Result<Option<Backend>, BackendRepoErr>;
    fn set(&self, game_id: &GameId, backend: Backend) -> Result<(), BackendRepoErr>;
}
