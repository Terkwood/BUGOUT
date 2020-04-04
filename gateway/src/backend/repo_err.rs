#[derive(Debug)]
pub enum BackendRepoErr {
    Redis(r2d2_redis::redis::RedisError),
}
impl From<r2d2_redis::redis::RedisError> for BackendRepoErr {
    fn from(r: r2d2_redis::redis::RedisError) -> Self {
        BackendRepoErr::Redis(r)
    }
}
