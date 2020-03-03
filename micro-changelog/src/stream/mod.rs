pub mod topics;
mod xread;

use crate::repo::entry_id_repo::EntryIdType;
use crate::repo::*;
use crate::Components;
use micro_model_moves::*;
use redis_streams::XReadEntryId;
pub use topics::StreamTopics;
use xread::*;
pub fn process(topics: StreamTopics, components: &crate::Components) {
    println!("Processing {:#?}", topics);
    loop {
        match entry_id_repo::fetch_all(components) {
            Ok(entry_ids) => match xread_sorted(entry_ids, &topics, &components.pool) {
                Ok(xrr) => {
                    for time_ordered_event in xrr {
                        match time_ordered_event {
                            (entry_id, StreamData::MA(move_acc)) => {
                                process_move_accepted(entry_id, move_acc, &components)
                            }
                            (entry_id, StreamData::GR(gr_ev)) => {
                                if let Err(e) = game_states_repo::write(
                                    gr_ev.game_id,
                                    GameState::default(), // TODO handicaps
                                    &components,
                                ) {
                                    println!("error saving fresh game state {:#?}", e)
                                } else {
                                    if let Err(e) = entry_id_repo::update(
                                        EntryIdType::GameReadyEvent,
                                        entry_id,
                                        &components,
                                    ) {
                                        println!("Error saving entry ID on game ready {:#?}", e)
                                    }
                                }
                            }
                            (entry_id, StreamData::GS(game_id, gs)) => {
                                if let Err(e) = game_states_repo::write(game_id, gs, &components) {
                                    println!("Error saving game state {:#?}", e)
                                } else {
                                    if let Err(e) = entry_id_repo::update(
                                        EntryIdType::GameStateChangelog,
                                        entry_id,
                                        &components,
                                    ) {
                                        println!("Error saving entry ID for game state {:#?}", e)
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("Redis err in xread: {:#?}", e),
            },
            Err(FetchErr::Deser) => println!("Unable to deserialize entry IDs"),
            Err(FetchErr::Redis(r)) => println!("Redis err {:#?}", r),
        }
    }
}

fn process_move_accepted(entry_id: XReadEntryId, move_acc: MoveMade, components: &Components) {
    let game_id = move_acc.game_id.clone();
    let old_game_state = game_states_repo::fetch(&move_acc.game_id, &components);
    let new_game_state = old_game_state.map(|mut og| {
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

        og.moves.push(move_acc);
        og
    });
    let state_saved = new_game_state.map(|gs| game_states_repo::write(game_id, gs, &components));
    if let Ok(Ok(write_result)) = state_saved {
        todo!("Announce to move made");
        entry_id_repo::update(EntryIdType::MoveAcceptedEvent, entry_id, &components);
    } else {
        println!("Trouble processing game state {:#?}", state_saved)
    }
}
