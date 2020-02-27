use super::conn_pool::Pool;
use super::redis_keys::RedisKeyNamespace;
use super::topics::*;
use super::xread::*;
use super::{FetchErr, WriteErr};
use crate::game::*;
use crate::model::*;
use crate::repo::entry_id::{EntryIdRepo, EntryIdType};
use crate::repo::game_states::GameStatesRepo;

pub fn process(opts: ProcessOpts, pool: &Pool) {
    let eid_repo = opts.entry_id_repo;
    loop {
        match eid_repo.fetch_all() {
            Ok(entry_ids) => {
                if let Ok(xread_result) = xread_sort(entry_ids, &opts.topics, &pool) {
                    for time_ordered_event in xread_result {
                        match time_ordered_event {
                            (entry_id, StreamData::MM(mm)) => {
                                match make_move(&mm, &opts.game_states_repo) {
                                    Err(e) => println!("Fetch err on game state {:#?}", e),
                                    Ok(Judgement::Accepted(move_made)) => {
                                        if let Err(e) = xadd_move_accepted(
                                            &move_made,
                                            &pool,
                                            &opts.topics.move_accepted_ev,
                                        ) {
                                            println!("Error XADD to move_accepted {:?}", e)
                                        } else {
                                            if let Err(e) = eid_repo
                                                .update(EntryIdType::MakeMoveCommand, entry_id)
                                            {
                                                println!(
                                                    "error updating make_move_cmd eid: {:?}",
                                                    e
                                                )
                                            }
                                        }
                                    }
                                    Ok(Judgement::Rejected) => println!("MOVE REJECTED: {:#?}", mm),
                                }
                            }
                            (entry_id, StreamData::GS(game_id, game_state)) => {
                                if let Err(e) = &opts.game_states_repo.write(game_id, game_state) {
                                    println!("error writing game state {:?}", e)
                                } else {
                                    if let Err(e) =
                                        eid_repo.update(EntryIdType::GameStateChangelog, entry_id)
                                    {
                                        println!("error updating changelog eid: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    println!("timeout")
                }
            }
            Err(FetchErr::Deser) => println!("Deserialization err in stream processing"),
            Err(FetchErr::Redis(r)) => println!("Redis error in stream processing {:?}", r),
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

fn make_move(mm: &MakeMoveCommand, gs_repo: &GameStatesRepo) -> Result<Judgement, FetchErr> {
    let game_state = gs_repo.fetch(&mm.game_id)?;
    Ok(judge(mm, &game_state))
}

fn xadd_move_accepted(
    move_made: &MoveMade,
    pool: &Pool,
    stream_name: &str,
) -> Result<String, WriteErr> {
    let mut conn = pool.get().unwrap();
    Ok(redis::cmd("XADD")
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
impl MoveMade {
    pub fn serialize(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        Ok(bincode::serialize(&self)?)
    }
}
