use crate::components::*;
use crate::*;
pub fn process(_topics: &StreamTopics, components: &Components) {
    loop {
        match components.entry_id_repo.fetch_all() {
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::components::Components;
    use crate::entry_id_repo::*;
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
        };
        todo!("write a unit test")
    }
}
