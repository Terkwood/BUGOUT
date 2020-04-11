use super::conn_pool::Pool;
use super::redis_keys::RedisKeyNamespace;
use super::topics::*;
use super::xread::*;
use super::{FetchErr, WriteErr};
use crate::game::*;
use crate::model::*;
use crate::repo::entry_id::{EntryIdRepo, EntryIdType};
use crate::repo::game_states::GameStatesRepo;

use log::{error, info, warn};

/// Spins too much.  See https://github.com/Terkwood/BUGOUT/issues/217
pub fn process(opts: ProcessOpts, pool: &Pool) {
    let eid_repo = opts.entry_id_repo;

    loop {
        match eid_repo.fetch_all() {
            Ok(entry_ids) => {
                if let Ok(xread_result) = xread_sort(&entry_ids, &opts.topics, &pool) {
                    for time_ordered_event in xread_result {
                        info!("EIDS {:?}", entry_ids);
                        match time_ordered_event {
                            (entry_id, StreamData::MM(mm)) => {
                                let fetched_gs = opts.game_states_repo.fetch(&mm.game_id);
                                match fetched_gs {
                                    Ok(Some(game_state)) => match judge(&mm, &game_state) {
                                        Judgement::Accepted(move_made) => {
                                            if let Err(e) = xadd_move_accepted(
                                                &move_made,
                                                &pool,
                                                &opts.topics.move_accepted_ev,
                                            ) {
                                                error!("Error XADD to move_accepted {:?}", e)
                                            }
                                        }
                                        Judgement::Rejected => {
                                            //err_count += 1; // TODO
                                            error!("MOVE REJECTED: {:#?}", mm)
                                        }
                                    },
                                    Ok(None) => warn!("No game state for game {}", mm.game_id.0),
                                    Err(e) => error!("Deser error ({:?})!", e),
                                }

                                if let Err(e) =
                                    eid_repo.update(EntryIdType::MakeMoveCommand, entry_id)
                                {
                                    error!("error updating make_move_cmd eid: {:?}", e)
                                }
                            }
                            (entry_id, StreamData::GS(game_id, game_state)) => {
                                if let Err(e) = &opts.game_states_repo.write(&game_id, &game_state)
                                {
                                    error!(
                                        "error writing game state {:?}  -- advancing eid pointer",
                                        e
                                    )
                                }

                                if let Err(e) =
                                    eid_repo.update(EntryIdType::GameStateChangelog, entry_id)
                                {
                                    error!("error updating changelog eid: {:?}", e);
                                }

                                info!("Tracking {:?} {:?}", game_id, game_state);
                            }
                        }
                    }
                }
            }
            Err(FetchErr::Deser) => error!("Deserialization err in stream processing"),
            Err(FetchErr::Redis(r)) => error!("Redis error in stream processing {:?}", r),
        }
    }
}

#[derive(Clone)]
pub struct ProcessOpts {
    pub topics: StreamTopics,
    pub entry_id_repo: EntryIdRepo,
    pub game_states_repo: GameStatesRepo,
}
impl Default for ProcessOpts {
    fn default() -> Self {
        let namespace = RedisKeyNamespace::default();
        let pool = super::conn_pool::create(super::conn_pool::RedisHostUrl::default());
        ProcessOpts {
            topics: StreamTopics::default(),
            entry_id_repo: EntryIdRepo {
                namespace: namespace.clone(),
                pool: pool.clone(),
            },
            game_states_repo: GameStatesRepo { namespace, pool },
        }
    }
}

fn xadd_move_accepted(
    move_made: &MoveMade,
    pool: &Pool,
    stream_name: &str,
) -> Result<String, WriteErr> {
    let mut conn = pool.get().unwrap();
    Ok(super::redis::cmd("XADD")
        .arg(stream_name)
        .arg("MAXLEN")
        .arg("~")
        .arg("1000")
        .arg("*")
        .arg("game_id")
        .arg(move_made.game_id.0.to_string())
        .arg("data")
        .arg(move_made.serialize()?)
        .query::<String>(&mut *conn)?)
}
