use super::RepoErr;
use redis::Commands;

const TTL_SECS: usize = 86400;
pub fn expire(key: &str, conn: &mut redis::Connection) -> Result<(), RepoErr> {
    Ok(conn.expire(key, TTL_SECS)?)
}
