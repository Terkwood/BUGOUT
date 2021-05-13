use super::topics;
use crate::Components;
use log::error;
use redis::Client;
use redis_stream::consumer::{Consumer, ConsumerOpts, Message};
use redis_streams::XReadEntryId as XID;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const BLOCK_MS: usize = 5000;
const GROUP_NAME: &str = "undo";
const CONSUMER_NAME: &str = "singleton";
const DATA_FIELD: &str = "data";

pub fn init(client: &Client, _components: Components) {
  let opts = || {
    ConsumerOpts::default()
      .group(GROUP_NAME, CONSUMER_NAME)
      .timeout(BLOCK_MS)
  };
  let unsorted: Arc<Mutex<HashMap<XID, StreamInput>>> = Arc::new(Mutex::new(HashMap::new()));

  let game_states_handler = |id: &str, message: &Message| {
    for (field, v) in message.iter() {
      if field.trim().to_lowercase() == DATA_FIELD {
        if let redis::Value::Data(bytes) = v {
          if let Some(stream_input) = bincode::deserialize(&bytes)
            .map(|gs| StreamInput::LOG(gs))
            .ok()
          {
            unsorted
              .lock()
              .expect("lock")
              .insert(XID::from_str(id).expect("xid deser"), stream_input.clone());
          }
        }
      }
    }

    Ok(())
  };
  let bot_attached_handler = |id: &str, message: &Message| {
    for (field, v) in message.iter() {
      if field.trim().to_lowercase() == DATA_FIELD {
        if let redis::Value::Data(bytes) = v {
          if let Some(stream_input) = bincode::deserialize(&bytes)
            .map(|ba| StreamInput::BA(ba))
            .ok()
          {
            unsorted
              .lock()
              .expect("lock")
              .insert(XID::from_str(id).expect("xid deser"), stream_input.clone());
          }
        }
      }
    }

    Ok(())
  };
  let undo_move_handler = |id: &str, message: &Message| {
    for (field, v) in message.iter() {
      if field.trim().to_lowercase() == DATA_FIELD {
        if let redis::Value::Data(bytes) = v {
          if let Some(stream_input) = bincode::deserialize(&bytes)
            .map(|um| StreamInput::UM(um))
            .ok()
          {
            unsorted
              .lock()
              .expect("lock")
              .insert(XID::from_str(id).expect("xid deser"), stream_input.clone());
          }
        }
      }
    }

    Ok(())
  };

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
      .expect("bot attached consumer init");
  let mut undo_move_consumer =
    Consumer::init(&mut c3, topics::UNDO_MOVE, undo_move_handler, opts())
      .expect("undo move consumer init");

  /*
  let xrr = conn.xread_options(
          &[UNDO_MOVE, BOT_ATTACHED, GAME_STATES_CHANGELOG],
          &[READ_OP, READ_OP, READ_OP],
          opts,
      )?;
      let unsorted = deser(xrr)?;

  */

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

    let mut u = unsorted.lock().expect("lock");
    let mut sorted_keys: Vec<XID> = u.keys().map(|k| *k).collect();
    sorted_keys.sort();
    let mut answer = vec![];
    for sk in sorted_keys {
      if let Some(data) = u.get(&sk) {
        answer.push((sk, data.clone()))
      }
    }

    u.clear();
  }
}

use super::undo::consume_undo;
use super::*;
use crate::repo::Botness;

fn consume_log(game_state: &GameState, reg: &Components) {
  if let Err(e) = reg.game_state_repo.put(&game_state) {
    error!("could not track game state: {:?}", e)
  }
}

fn consume_bot_attached(ba: &BotAttached, reg: &Components) {
  if let Err(e) = reg.botness_repo.put(&ba.game_id, ba.player, Botness::IsBot) {
    error!("could not track bot attached: {:?}", e)
  }
}
