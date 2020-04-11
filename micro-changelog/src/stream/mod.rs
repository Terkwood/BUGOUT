pub mod topics;
mod xread;

use crate::repo::entry_id_repo::EntryIdType;
use crate::repo::*;
use crate::Components;
use micro_model_moves::*;
use redis_conn_pool::redis;
pub use topics::StreamTopics;
use xread::*;

use log::{error, info};

pub fn process(topics: StreamTopics, components: &crate::Components) {
    info!("Processing {:#?}", topics);
    loop {
        match entry_id_repo::fetch_all(components) {
            Ok(entry_ids) => match xread_sorted(entry_ids, &topics, &components.pool) {
                Ok(xrr) => {
                    for time_ordered_event in xrr {
                        match time_ordered_event {
                            (entry_id, StreamData::MA(move_acc)) => {
                                match update_game_state(&move_acc, &components) {
                                    Err(e) => error!("err updating game state {:?}", e),
                                    Ok(gs) => {
                                        // These next two ops are concurrent in the kafka impl
                                        if let Err(e) = xadd_game_states_changelog(
                                            &move_acc.game_id,
                                            gs,
                                            &topics.game_states_changelog,
                                            components,
                                        ) {
                                            error!("could not XADD to game state changelog {:?}", e)
                                        } 

                                        if let Err(e) = xadd_move_made(
                                            &move_acc,
                                            &topics.move_made_ev,
                                            &components,
                                        ) {
                                            error!("err in XADD move made {:?}", e)
                                        }

                                        if let Err(e) = entry_id_repo::update(
                                            EntryIdType::MoveAcceptedEvent,
                                            entry_id,
                                            &components,
                                        ) {
                                            error!("err saving entry id for move accepted {:?}", e)
                                        }
                                    }
                                }
                            }
                            (entry_id, StreamData::GS(game_id, gs)) => {
                                if let Err(e) = game_states_repo::write(&game_id, &gs, &components)
                                {
                                    error!("Error saving game state {:#?}", e)
                                } else {
                                    info!("wrote game state: {:?} {:?}", game_id, gs);
                                }

                                if let Err(e) = entry_id_repo::update(
                                    EntryIdType::GameStateChangelog,
                                    entry_id,
                                    &components,
                                ) {
                                    error!("Error saving entry ID for game state {:#?}", e)
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("Redis err in xread: {:#?}", e),
            },
            Err(FetchErr::Deser) => error!("Unable to deserialize entry IDs"),
            Err(FetchErr::Redis(r)) => error!("Redis err {:#?}", r),
        }
    }
}

fn update_game_state(
    move_acc: &MoveMade,
    components: &Components,
) -> Result<GameState, GameStateSaveErr> {
    let game_id = move_acc.game_id.clone();
    let old_game_state = game_states_repo::fetch(&move_acc.game_id, &components);
    let new_game_state = old_game_state.map(|maybe_og| {
        let mut og = maybe_og.unwrap_or(GameState::default());
        og.turn += 1;
        og.player_up = match move_acc.player {
            Player::BLACK => Player::WHITE,
            _ => Player::BLACK,
        };
        let caps = move_acc.captured.len() as u16;
        match move_acc.player {
            Player::BLACK => og.captures.black += caps,
            Player::WHITE => og.captures.white += caps,
        }
        for c in &move_acc.captured {
            og.board.pieces.remove(c);
        }
        if let Some(c) = &move_acc.coord {
            og.board.pieces.insert(*c, move_acc.player);
        }

        og.moves.push(move_acc.clone());
        og
    })?;
    game_states_repo::write(&game_id, &new_game_state, &components)?;
    Ok(new_game_state)
}

#[derive(Debug)]
enum GameStateSaveErr {
    W(WriteErr),
    F(FetchErr),
}
impl From<WriteErr> for GameStateSaveErr {
    fn from(w: WriteErr) -> Self {
        GameStateSaveErr::W(w)
    }
}
impl From<FetchErr> for GameStateSaveErr {
    fn from(f: FetchErr) -> Self {
        GameStateSaveErr::F(f)
    }
}

fn xadd_move_made(
    mm: &MoveMade,
    stream_name: &str,
    components: &Components,
) -> Result<String, WriteErr> {
    let mut conn = components.pool.get().unwrap();
    Ok(redis::cmd("XADD")
        .arg(stream_name)
        .arg("MAXLEN")
        .arg("~")
        .arg("1000")
        .arg("*")
        .arg("game_id")
        .arg(mm.game_id.0.to_string())
        .arg("data")
        .arg(mm.serialize()?)
        .query::<String>(&mut *conn)?)
}

fn xadd_game_states_changelog(
    game_id: &GameId,
    gs: GameState,
    stream_name: &str,
    components: &Components,
) -> Result<String, WriteErr> {
    let mut conn = components.pool.get().unwrap();
    info!("Must XADD game state: {:?}", gs);
    Ok(redis::cmd("XADD")
        .arg(stream_name)
        .arg("MAXLEN")
        .arg("~")
        .arg("1000")
        .arg("*")
        .arg("game_id")
        .arg(game_id.0.to_string())
        .arg("data")
        .arg(gs.serialize()?)
        .query::<String>(&mut *conn)?)
}
