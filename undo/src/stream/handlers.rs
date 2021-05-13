use super::topics;
use super::GROUP_NAME;
use crate::Components;
use log::error;
use redis::Client;
use redis_stream::consumer::{Consumer, ConsumerOpts, Message};

const BLOCK_MS: usize = 5000;
const CONSUMER_NAME: &str = "singleton";

pub fn init(client: &Client, _components: Components) {
    let opts = || {
        ConsumerOpts::default()
            .group(GROUP_NAME, CONSUMER_NAME)
            .timeout(BLOCK_MS)
    };

    let game_states_handler = |_id: &str, _message: &Message| Ok(todo!());
    let bot_attached_handler = |_id: &str, _message: &Message| Ok(todo!());
    let undo_move_handler = |_id: &str, _message: &Message| Ok(todo!());

    let conn = || client.get_connection().expect("conn");
    let mut c1 = conn();
    let mut c2 = conn();
    let mut c3 = conn();
    let mut game_states_consumer = Consumer::init(
        &mut c1,
        topics::GAME_STATES_CHANGELOG,
        game_states_handler,
        opts(),
    )
    .expect("game states consumer init");
    let mut bot_attached_consumer =
        Consumer::init(&mut c2, topics::BOT_ATTACHED, bot_attached_handler, opts())
            .expect("game states consumer init");
    let mut undo_move_consumer =
        Consumer::init(&mut c3, topics::UNDO_MOVE, undo_move_handler, opts())
            .expect("undo move consumer init");

    loop {
        if let Err(e) = game_states_consumer.consume() {
            error!("could not consume game states stream {:?}", e)
        }
        if let Err(e) = bot_attached_consumer.consume() {
            error!("could not consume BOT ATTACHED stream {:?}", e)
        }
        if let Err(e) = undo_move_consumer.consume() {
            error!("could not consume UNDO MOVE stream {:?}", e)
        }
    }
}
