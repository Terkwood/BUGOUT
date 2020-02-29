pub mod topics;
mod xread;

use crate::repo::*;
pub use topics::StreamTopics;
use xread::xread_sorted;

pub fn process(topics: StreamTopics, components: &crate::Components) {
    println!("Processing {:#?}", topics);
    loop {
        match entry_id_repo::fetch_all(components) {
            Ok(entry_ids) => {
                if let Ok(xread_result) = xread_sorted(entry_ids, &topics, &components.pool) {
                    todo!()
                } else {
                    todo!()
                }
            }
            Err(_) => todo!(),
        }
    }
}
