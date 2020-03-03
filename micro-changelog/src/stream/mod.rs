pub mod topics;
mod xread;

use crate::repo::*;
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
                            (entry_id, StreamData::MA(move_acc)) => todo!(),
                            (entry_id, StreamData::GR(gr_ev)) => todo!(),
                            (entry_id, StreamData::GS(game_id, gs)) => todo!(),
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
