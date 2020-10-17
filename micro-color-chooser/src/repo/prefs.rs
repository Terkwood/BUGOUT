use super::*;
use core_model::SessionId;
use log::trace;

pub trait PrefsRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr>;
    fn put(&self, scp: &SessionColorPref) -> Result<(), WriteErr>;
}

impl PrefsRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr> {
        trace!("get {:?}", &session_id);
        match self.get_connection() {
            Ok(mut conn) => {
                let key = redis_key(session_id);
                let data: Option<Vec<u8>> = conn.get(&key)?;

                if let Some(bytes) = data {
                    touch_ttl(&mut conn, &key);
                    match bincode::deserialize(&bytes) {
                        Ok(game_ready) => Ok(Some(game_ready)),
                        Err(e) => Err(FetchErr::Deser(e)),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(FetchErr::Redis(e)),
        }
    }

    fn put(&self, scp: &SessionColorPref) -> Result<(), WriteErr> {
        let c = self.get_connection();
        let s = bincode::serialize(scp);
        if let (Ok(mut conn), Ok(bytes)) = (c, s) {
            let key = redis_key(&scp.session_id);

            trace!("Write serialized bytes {:?}", &bytes);
            let done: Result<(), _> = conn.set(&key, bytes.clone());
            if let Ok(_) = done {
                touch_ttl(&mut conn, &key)
            }

            done.map_err(|_| WriteErr)
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(session_id: &SessionId) -> String {
    format!("/BUGOUT/micro_color_chooser/prefs/{}", session_id.0)
}
