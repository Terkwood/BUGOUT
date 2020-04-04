use super::{Backend,BackendRepoErr};
use crate::model::ClientId;

pub trait ClientBackendRepo {
    fn get(&self, client_id: &ClientId) -> Result<Option<Backend>, BackendRepoErr>;
    fn set(&self, client_id: &ClientId, backend:Backend) -> Result<(), BackendRepoErr>;
}
