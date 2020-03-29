use super::*;
use crate::model::SessionId;
use crate::redis_io::KeyProvider;
use r2d2_redis::r2d2;
use r2d2_redis::r2d2::Pool;
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;

pub trait ClientBackendRepo {
    fn backend_for(&self, session_id: &SessionId) -> Result<Option<Backend>, BackendRepoErr>;

    fn assign(&mut self, session_id: &SessionId, backend: Backend) -> Result<(), BackendRepoErr>;

    fn unassign_all(&mut self, backend: Backend) -> Result<(), BackendRepoErr>;
}

const TTL_SECS: usize = 86400;

pub struct RedisClientBackendRepo {
    pub pool: Pool<RedisConnectionManager>,
    pub key_provider: KeyProvider,
}

impl ClientBackendRepo for RedisClientBackendRepo {
    fn backend_for(&self, session_id: &SessionId) -> Result<Option<Backend>, BackendRepoErr> {
        let mut conn = self.pool.get().expect("pool");

        Ok(
            if conn.sismember(
                self.key_provider.backend(Backend::RedisStreams),
                session_id.to_string(),
            )? {
                self.expire(Backend::RedisStreams, &mut conn)?;
                Some(Backend::RedisStreams)
            } else if conn.sismember(
                self.key_provider.backend(Backend::Kafka),
                session_id.to_string(),
            )? {
                self.expire(Backend::Kafka, &mut conn)?;
                Some(Backend::Kafka)
            } else {
                None
            },
        )
    }

    fn assign(&mut self, session_id: &SessionId, backend: Backend) -> Result<(), BackendRepoErr> {
        let mut conn = self.pool.get().expect("pool");

        let result = conn.sadd(self.key_provider.backend(backend), session_id.to_string())?;
        self.expire(backend, &mut conn)?;
        Ok(result)
    }

    fn unassign_all(&mut self, backend: Backend) -> std::result::Result<(), BackendRepoErr> {
        let mut conn = self.pool.get().expect("pool");

        Ok(conn.del(self.key_provider.backend(backend))?)
    }
}
impl RedisClientBackendRepo {
    fn expire(
        &self,
        backend: Backend,
        conn: &mut r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>,
    ) -> Result<(), BackendRepoErr> {
        Ok(conn.expire(self.key_provider.backend(backend), TTL_SECS)?)
    }
}
#[derive(Debug)]
pub enum BackendRepoErr {
    Redis(r2d2_redis::redis::RedisError),
}
impl From<r2d2_redis::redis::RedisError> for BackendRepoErr {
    fn from(r: r2d2_redis::redis::RedisError) -> Self {
        BackendRepoErr::Redis(r)
    }
}
