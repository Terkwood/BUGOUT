use super::*;
use core_model::*;
use redis::Client;
use std::rc::Rc;
use sync_model::api::ReqSync;

/// "Do we need to form a reply?"
/// Used when client is ahead of the system.  Stores
/// a requested sync event which can later be merged
/// with a MOVE MADE event to form a sync reply.
pub trait ReplyOnMoveRepo {
    fn get(&self, game_id: &GameId, req_id: &ReqId) -> Result<Option<ReqSync>, FetchErr>;
    fn put(&self, req: &ReqSync) -> Result<(), WriteErr>;
    fn del(&self, game_id: &GameId, req_id: &ReqId) -> Result<(), WriteErr>;
}

impl ReplyOnMoveRepo for Rc<Client> {
    fn get(&self, game_id: &GameId, req_id: &ReqId) -> Result<Option<ReqSync>, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let key = redis_key(game_id, req_id);
            let data: Result<Vec<u8>, _> = conn.get(&key).map_err(|_| FetchErr);

            if data.is_ok() {
                touch_ttl(&mut conn, &key)
            }

            data.and_then(|bytes| bincode::deserialize(&bytes).map_err(|_| FetchErr))
        } else {
            Err(FetchErr)
        }
    }

    fn put(&self, req: &ReqSync) -> Result<(), WriteErr> {
        let key = redis_key(&req.game_id, &req.req_id);
        if let (Ok(mut conn), Ok(bytes)) = (self.get_connection(), bincode::serialize(&req)) {
            let done = conn.set(&key, bytes).map_err(|_| WriteErr)?;
            touch_ttl(&mut conn, &key);
            Ok(done)
        } else {
            Err(WriteErr)
        }
    }

    fn del(&self, game_id: &GameId, req_id: &ReqId) -> Result<(), WriteErr> {
        let key = redis_key(&game_id, &req_id);
        if let Ok(mut conn) = self.get_connection() {
            conn.del(&key).map_err(|_| WriteErr)
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(game_id: &GameId, req_id: &ReqId) -> String {
    format!(
        "/BUGOUT/micro_sync/reply_on_move/{}_{}",
        game_id.0, req_id.0
    )
}
