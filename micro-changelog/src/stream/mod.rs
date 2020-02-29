pub mod topics;
use crate::repo::*;
pub use topics::StreamTopics;
pub fn process(topics: StreamTopics, components: &crate::Components) {
    println!("Processing {:#?}", topics);
    loop {}
}
