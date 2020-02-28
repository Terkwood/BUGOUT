pub mod topics;
use crate::repo::game_states::GameStatesRepo;
pub use topics::StreamTopics;
pub fn process(topics: StreamTopics, game_states_repo: GameStatesRepo) {
    println!("ay");
    println!("Processing {:#?} with {:#?}", topics, game_states_repo);
}
