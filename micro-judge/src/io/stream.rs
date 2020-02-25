use super::conn_pool::Pool;
use super::redis_keys::Namespace;
use super::topics::*;
use super::xread::*;
use super::FetchErr;
use crate::game::validate_move;
use crate::model::*;
use crate::repo::entry_id::{EntryIdRepo, EntryIdType};
use crate::repo::game_states;

pub fn process(opts: ProcessOpts, pool: &Pool) {
    let eid_repo = opts.entry_id_repo;
    loop {
        match eid_repo.fetch_all() {
            Ok(entry_ids) => {
                if let Ok(xread_result) = xread_sort(entry_ids, &opts.topics, &pool) {
                    for time_ordered_event in xread_result {
                        match time_ordered_event {
                            (_entry_id, StreamData::MM(mm)) => {
                                make_move(mm, &opts.namespace, &pool)
                            }
                            (entry_id, StreamData::GS(game_id, game_state)) => {
                                if let Err(e) =
                                    game_states::write(game_id, game_state, &opts.namespace, &pool)
                                {
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
            Err(FetchErr::Deser) => todo!("deser"),
            Err(FetchErr::Redis(r)) => todo!("redis {}", r),
        }
    }
}
#[derive(Clone)]
pub struct ProcessOpts {
    pub topics: StreamTopics,
    pub namespace: Namespace,
    pub entry_id_repo: EntryIdRepo,
}
impl Default for ProcessOpts {
    fn default() -> Self {
        let namespace = Namespace::default();
        ProcessOpts {
            topics: StreamTopics::default(),
            namespace: namespace.clone(),
            entry_id_repo: EntryIdRepo {
                namespace,
                pool: super::conn_pool::create(super::conn_pool::RedisHostUrl::default()),
            },
        }
    }
}

fn make_move(mm: MakeMoveCommand, ns: &Namespace, pool: &Pool) {
    if let Ok(game_state) = game_states::fetch(&mm.game_id, ns, &pool) {
        if validate_move(mm, game_state) {
            todo!()
        } else {
            println!("Invalid move")
        }
    } else {
        println!("failed to fetch game state repo")
    }
}
