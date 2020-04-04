use super::*;
use crate::model::SessionId;
use crate::redis_io::{KeyProvider, RedisPool};

use r2d2_redis::r2d2;
use r2d2_redis::r2d2::Pool;
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;

pub trait SessionBackendRepo {
    fn backend_for(&self, session_id: &SessionId) -> Result<Option<Backend>, BackendRepoErr>;

    fn assign(&self, session_id: &SessionId, backend: Backend) -> Result<(), BackendRepoErr>;

    fn unassign(&self, session_id: &SessionId) -> Result<(), BackendRepoErr>;

    fn unassign_all(&mut self, backend: Backend) -> Result<(), BackendRepoErr>;
}

pub fn create(pool: RedisPool) -> Box<dyn SessionBackendRepo> {
    Box::new(RedisSessionBackendRepo {
        key_provider: KeyProvider::default(),
        pool,
    })
}

const TTL_SECS: usize = 86400;

pub struct RedisSessionBackendRepo {
    pub pool: Pool<RedisConnectionManager>,
    pub key_provider: KeyProvider,
}

impl SessionBackendRepo for RedisSessionBackendRepo {
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

    fn assign(&self, session_id: &SessionId, backend: Backend) -> Result<(), BackendRepoErr> {
        let mut conn = self.pool.get().expect("pool");

        let result = conn.sadd(self.key_provider.backend(backend), session_id.to_string())?;
        self.expire(backend, &mut conn)?;
        Ok(result)
    }

    fn unassign(&self, session_id: &SessionId) -> Result<(), BackendRepoErr> {
        let mut conn = self.pool.get().expect("pool");

        conn.srem(
            self.key_provider.backend(Backend::RedisStreams),
            session_id.to_string(),
        )?;
        Ok(conn.srem(
            self.key_provider.backend(Backend::Kafka),
            session_id.to_string(),
        )?)
    }

    fn unassign_all(&mut self, backend: Backend) -> std::result::Result<(), BackendRepoErr> {
        let mut conn = self.pool.get().expect("pool");

        Ok(conn.del(self.key_provider.backend(backend))?)
    }
}
impl RedisSessionBackendRepo {
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
