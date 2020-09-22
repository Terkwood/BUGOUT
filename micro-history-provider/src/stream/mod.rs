mod topics;
mod xadd;
mod xread;

pub use xadd::*;
pub use xread::*;

use crate::api::*;
use crate::components::*;
use crate::model::*;
use log::warn;
use redis::Commands;

#[derive(Clone, Debug)]
pub enum StreamInput {
    PH(ProvideHistory),
    GS(GameId, GameState),
}

pub fn process(components: &Components) {
    loop {
        todo!()
    }
}

const GROUP_NAME: &str = "micro-history-provider";

pub fn create_consumer_group(client: &redis::Client) {
    let mut conn = client.get_connection().expect("group create conn");
    let mm: Result<(), _> = conn.xgroup_create_mkstream(topics::PROVIDE_HISTORY, GROUP_NAME, "$");
    if let Err(e) = mm {
        warn!(
            "Ignoring error creating {} consumer group (it probably exists already) {:?}",
            topics::PROVIDE_HISTORY,
            e
        );
    }
    let gs: Result<(), _> =
        conn.xgroup_create_mkstream(topics::GAME_STATES_CHANGELOG, GROUP_NAME, "$");
    if let Err(e) = gs {
        warn!(
            "Ignoring error creating {} consumer group (it probably exists already) {:?}",
            topics::GAME_STATES_CHANGELOG,
            e
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repo::*;
    use crate::Components;
    use crossbeam_channel::{select, unbounded, Receiver, Sender};
    use redis_streams::XReadEntryId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    static FAKE_PROV_HIST_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_GAME_STATES_MILLIS: AtomicU64 = AtomicU64::new(0);
    static FAKE_PROV_HIST_SEQ: AtomicU64 = AtomicU64::new(0);
    static FAKE_GAME_STATES_SEQ: AtomicU64 = AtomicU64::new(0);

    struct FakeHistoryRepo {
        pub contents: Arc<Mutex<Option<Vec<Move>>>>,
        pub put_in: Sender<Vec<Move>>,
    }

    impl HistoryRepo for FakeHistoryRepo {
        fn get(&self, game_id: GameId) -> Result<Option<Vec<Move>>, FetchErr> {
            Ok(self.contents.lock().expect("mutex").clone())
        }

        fn put(&self, game_id: GameId, moves: Vec<Move>) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("mutex");
            *data = Some(moves.clone());
            Ok(self.put_in.send(moves).expect("send"))
        }
    }

    struct FakeXRead {
        sorted_data: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>>,
    }
    impl XRead for FakeXRead {
        fn xread_sorted(
            &self,
        ) -> Result<Vec<(redis_streams::XReadEntryId, StreamInput)>, redis::RedisError> {
            todo!()
        }
    }

    struct FakeXAdd(Sender<HistoryProvided>);
    impl XAdd for FakeXAdd {
        fn xadd(&self, data: HistoryProvided) -> Result<(), XAddErr> {
            Ok(self.0.send(data).expect("send"))
        }
    }

    #[test]
    fn test_process() {
        let (xadd_call_in, xadd_call_out): (Sender<HistoryProvided>, _) = unbounded();
        let (put_history_in, put_history_out): (Sender<Vec<Move>>, _) = unbounded();

        // set up a loop to process game lobby requests
        let fake_history_contents = Arc::new(Mutex::new(None));
        let fh = fake_history_contents.clone();
        std::thread::spawn(move || loop {
            select! {
                recv(put_history_out) -> msg => match msg {
                    Ok(moves) => *fh.lock().expect("mutex lock") = Some(moves),
                    Err(_) => panic!("fail")
                }
            }
        });

        let sorted_fake_stream: Arc<Mutex<Vec<(XReadEntryId, StreamInput)>>> =
            Arc::new(Mutex::new(vec![]));

        let sfs = sorted_fake_stream.clone();
        let fh = fake_history_contents.clone();
        thread::spawn(move || {
            let components = Components {
                history_repo: Box::new(FakeHistoryRepo {
                    contents: fh,
                    put_in: put_history_in,
                }),
                xread: Box::new(FakeXRead {
                    sorted_data: sfs.clone(),
                }),
                xadd: Box::new(FakeXAdd(xadd_call_in)),
            };
            process(&components);
        });

        // emit some events in a time-ordered fashion
        // (fake xread impl expects time ordering 😁)

        let timeout = Duration::from_millis(166);
        let mut fake_time_ms = 100;
        let incr_ms = 100;

        let fake_game_id = GameId(uuid::Uuid::default());
        let fake_moves = vec![
            MoveEvent {
                player: Player::BLACK,
                coord: Some(Coord { x: 1, y: 1 }),
            },
            MoveEvent {
                player: Player::WHITE,
                coord: None,
            },
        ];
        let fake_player_up = Player::BLACK;
        // emit a game state
        sorted_fake_stream.lock().expect("lock").push((
            todo!("quick eid"),
            StreamInput::GS(
                fake_game_id.clone(),
                GameState {
                    moves: Some(fake_moves),
                    player_up: fake_player_up,
                },
            ),
        ));

        thread::sleep(timeout);

        // history repo should now contain the moves from that game
        let actual_moves = fake_history_contents.clone().lock().expect("hr").unwrap();
        assert_eq!(actual_moves.len(), 1)
    }
}
