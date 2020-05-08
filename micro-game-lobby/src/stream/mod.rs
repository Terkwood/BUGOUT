mod xread;

pub use xread::*;

use crate::components::Components;
use crate::*;
use log::error;

pub fn process(topics: &topics::StreamTopics, components: &Components) {
    loop {
        match components.entry_id_repo.fetch_all() {
            Ok(xrr) => match components.xreader.xread_sorted(todo!(), topics) {
                Ok(xrr) => {
                    for time_ordered_event in xrr {
                        todo!()
                    }
                }
                Err(e) => error!("Stream err {}", e),
            },
            Err(e) => error!("Failed to fetch EIDs"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::components::Components;
    use crate::repo::*;
    struct FakePool;
    impl EntryIdRepo for FakePool {
        fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
            unimplemented!()
        }
    }
    #[test]
    fn test_process() {
        let components = Components {
            entry_id_repo: Box::new(FakePool),
            xreader: todo!(),
        };
        todo!("write a unit test")
    }
}
