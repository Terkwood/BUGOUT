use super::RedisKeyNamespace;
use crate::backend::Backend;
#[derive(Debug, Clone)]
pub struct KeyProvider(pub RedisKeyNamespace);
impl Default for KeyProvider {
    fn default() -> Self {
        KeyProvider(RedisKeyNamespace::default())
    }
}

impl KeyProvider {
    pub fn entry_ids(&self) -> String {
        format!("/{}/gateway/entry_ids", (self.0).0)
    }

    pub fn backend(&self, backend: Backend) -> String {
        format!("/{}/gateway/backend/{}", (self.0).0, backend)
    }
}
