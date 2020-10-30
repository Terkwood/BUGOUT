use super::{expire, RepoErr};
use bot_model::Bot;
use core_model::GameId;
use move_model::Player;
use redis::{Client, Commands};
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attachment {
    pub bot: Bot,
    pub player: Player,
    pub game_id: GameId,
}

pub trait AttachmentRepo: Send + Sync {
    fn get(&self, game_id: &GameId, player: Player) -> Result<Option<Attachment>, RepoErr>;
    fn put(&self, attachment: &Attachment) -> Result<(), RepoErr>;
}

impl AttachmentRepo for Arc<Client> {
    fn get(&self, game_id: &GameId, player: Player) -> Result<Option<Attachment>, RepoErr> {
        match self.get_connection() {
            Ok(mut conn) => {
                let key = attachment_id(game_id, player);
                let data: Result<Option<Vec<u8>>, _> =
                    conn.get(&key).map_err(|e| RepoErr::Redis(e));

                if data.is_ok() {
                    expire(&key, &mut conn)?
                }

                match data {
                    Ok(Some(bytes)) => {
                        let deser: Result<Attachment, _> = bincode::deserialize(&bytes);
                        deser.map(|d| Some(d)).map_err(|e| RepoErr::SerDes(e))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(RepoErr::Redis(e)),
        }
    }

    fn put(&self, attachment: &Attachment) -> Result<(), RepoErr> {
        let key = attachment_id(&attachment.game_id, attachment.player);
        let mut conn = self.get_connection()?;
        let bytes = bincode::serialize(&attachment)?;
        let done = conn.set(&key, bytes).map_err(|e| RepoErr::Redis(e))?;
        expire(&key, &mut conn)?;
        Ok(done)
    }
}

fn attachment_id(game_id: &GameId, player: Player) -> String {
    format!(
        "/BUGOUT/botlink/attachment/{}_{}",
        game_id.0.to_string(),
        player.to_string()
    )
}
