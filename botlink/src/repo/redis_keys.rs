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
        format!("/{}/botlink/entry_ids", (self.0).0)
    }
    pub fn attached_bots(&self) -> String {
        format!("/{}/botlink/attached_bots", (self.0).0)
    }
}
