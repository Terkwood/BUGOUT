use crate::entry_id_repo::EntryIdRepo;
pub struct Components {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
}
