use std::collections::HashMap;
use std::ops::Add;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam::{Receiver, Sender};
use crossbeam_channel::select;

use uuid::Uuid;

use crate::logging::{short_uuid, EMPTY_SHORT_UUID, MEGA_DEATH_STRING};
use crate::model::{ClientId, Events, GameId, OpenGameReplyEvent, Player, ReconnectedEvent, ReqId};

const GAME_STATE_CLEANUP_PERIOD_MS: u64 = 10_000;

/// start the select! loop responsible for sending kafka messages to relevant websocket clients
/// it must respond to requests to let it add and drop listeners
pub fn start(router_commands_out: Receiver<RouterCommand>, kafka_events_out: Receiver<Events>) {
    thread::spawn(move || {
        let mut router = Router::new();
        loop {
            select! {
                recv(router_commands_out) -> command =>
                    match command {
                        Ok(RouterCommand::Observe(game_id)) => router.observed(game_id),
                        Ok(RouterCommand::RequestOpenGame{client_id, events_in, req_id}) => {
                            let game_id = router.add_client(client_id, events_in.clone());
                            if let Err(err) = events_in.send(Events::OpenGameReply(OpenGameReplyEvent{game_id, reply_to:req_id, event_id: Uuid::new_v4()})) {
                                println!("😯 {} {} {:<8} could not send open game reply {}",
                                    short_uuid(client_id), EMPTY_SHORT_UUID, "ERROR", err)
                            }
                        },
                        Ok(RouterCommand::DeleteClient{client_id, game_id}) =>
                            router.delete_client(client_id, game_id),
                        Ok(RouterCommand::RegisterOpenGame{game_id}) =>
                            router.register_open_game(game_id),
                        Ok(RouterCommand::Reconnect{client_id, game_id, events_in, req_id }) => {
                            router.reconnect_client(client_id, game_id, events_in.clone());
                            if let Err(err) = events_in.send(Events::Reconnected(ReconnectedEvent{game_id, reply_to: req_id, event_id: Uuid::new_v4(), player_up: router.playerup(game_id)})) {
                                println!(
                                    "😦 {} {} {:<8} could not send reconnect reply {}",
                                    short_uuid(client_id),
                                    short_uuid(game_id),
                                    "ERROR", err)
                            }
                        },
                        Err(e) => panic!("Unable to receive command via router channel: {:?}", e),
                    },
                recv(kafka_events_out) -> event =>
                    match event {
                        Ok(Events::MoveMade(m)) => {
                            let u = m.clone();
                            router.set_playerup(u.game_id, u.player.other());
                            router.forward_event(Events::MoveMade(m))
                        }
                        Ok(e) => {
                            router.observed(e.game_id());
                            router.forward_event(e)},
                        Err(e) =>
                            panic!("Unable to receive kafka event via router channel: {:?}", e),
                    }
            }
        }
    });
}

struct GameState {
    pub clients: Vec<ClientSender>,
    pub playerup: Player,
    pub modified_at: Instant,
}

impl GameState {
    pub fn new(client: ClientSender) -> GameState {
        GameState {
            clients: vec![client],
            playerup: Player::BLACK,
            modified_at: Instant::now(),
        }
    }

    pub fn add_client(&mut self, client: ClientSender) {
        self.clients.push(client);
        self.modified_at = Instant::now()
    }

    /// Observe that this game still has an open connection somewhere
    pub fn observed(&mut self) {
        self.modified_at = Instant::now()
    }
}

/// Keeps track of clients interested in various games
/// Each client has an associated crossbeam Sender for BUGOUT events
struct Router {
    pub available_games: Vec<GameId>,
    pub game_states: HashMap<GameId, GameState>,
    pub last_cleanup: Instant,
}

impl Router {
    pub fn new() -> Router {
        Router {
            available_games: vec![],
            game_states: HashMap::new(),
            last_cleanup: Instant::now(),
        }
    }

    pub fn forward_event(&self, ev: Events) {
        if let Some(GameState {
            clients,
            playerup: _,
            modified_at: _,
        }) = self.game_states.get(&ev.game_id())
        {
            for c in clients {
                if let Err(err) = c.events_in.send(ev.clone()) {
                    println!(
                        "😑 {} {} {:<8} forwarding event {}",
                        short_uuid(c.client_id),
                        short_uuid(ev.game_id()),
                        "ERROR",
                        err
                    )
                }
            }
        }
    }

    pub fn playerup(&self, game_id: GameId) -> Player {
        self.game_states
            .get(&game_id)
            .map(|game_state| &game_state.playerup)
            .unwrap_or(&Player::BLACK)
            .clone()
    }

    /// Note that the game state somehow changed, so that
    /// we don't purge it prematurely in cleanup_game_states()
    pub fn observed(&mut self, game_id: GameId) {
        self.game_states.get_mut(&game_id).map(|gs| gs.observed());
    }

    pub fn set_playerup(&mut self, game_id: GameId, player: Player) {
        let c = player.clone();
        let default = GameState {
            clients: vec![],
            playerup: c,
            modified_at: Instant::now(),
        };
        let gs = self.game_states.get_mut(&game_id);
        match gs {
            None => {
                self.game_states.insert(game_id, default);
            }
            Some(gs) => {
                gs.playerup = player;
                gs.observed();
            }
        }
    }

    pub fn add_client(&mut self, client_id: ClientId, events_in: Sender<Events>) -> GameId {
        let newbie = ClientSender {
            client_id,
            events_in,
        };

        let game_id = self.pop_open_game_id();
        match self.game_states.get_mut(&game_id) {
            Some(gs) => gs.add_client(newbie),
            None => {
                let with_newbie = GameState::new(newbie);
                self.game_states.insert(game_id, with_newbie);
            }
        }

        game_id
    }

    pub fn reconnect_client(
        &mut self,
        client_id: ClientId,
        game_id: GameId,
        events_in: Sender<Events>,
    ) {
        let cs = ClientSender {
            client_id,
            events_in,
        };

        match self.game_states.get_mut(&game_id) {
            Some(gs) => gs.add_client(cs),
            None => {
                self.game_states.insert(game_id, GameState::new(cs));
            }
        }
    }

    fn find_dead_game_states(&mut self) -> Vec<GameId> {
        let mut to_delete = vec![];

        for (game_id, game_state) in self.game_states.iter() {
            if game_state.clients.len() == 0 {
                let since = Instant::now().checked_duration_since(
                    game_state
                        .modified_at
                        .add(Duration::from_millis(GAME_STATE_CLEANUP_PERIOD_MS)),
                );

                if let Some(dur) = since {
                    if dur.as_millis() > GAME_STATE_CLEANUP_PERIOD_MS.into() {
                        // we will destroy the game state if there are
                        // no clients connected to it
                        to_delete.push(*game_id);
                    }
                }
            }
        }

        to_delete
    }
    fn cleanup_game_states(&mut self) {
        let since = Instant::now().checked_duration_since(self.last_cleanup);
        if let Some(dur) = since {
            if dur.as_millis() > GAME_STATE_CLEANUP_PERIOD_MS.into() {
                let to_delete = self.find_dead_game_states();
                let mut count = 0;
                for game_id in to_delete {
                    self.game_states.remove_entry(&game_id);

                    count += 1;
                }

                self.last_cleanup = Instant::now();
                if count > 0 {
                    println!(
                        "🗑 {} {} {:<8} {:<4} entries",
                        EMPTY_SHORT_UUID, EMPTY_SHORT_UUID, "CLEANUP", count
                    )
                }
            }
        }
    }

    pub fn delete_client(&mut self, client_id: ClientId, game_id: GameId) {
        if let Some(game_state) = self.game_states.get_mut(&game_id) {
            game_state.clients.retain(|c| c.client_id != client_id);
        }

        self.cleanup_game_states();
    }

    /// This is a big fat hack...
    /// We want the game IDs to be data driven via kafka.
    /// But this is better than having the game IDs hardcoded.
    pub fn register_open_game(&mut self, game_id: GameId) {
        // Register duplicates as we'll plan to consume two at a time
        self.available_games.push(game_id);
        self.available_games.push(game_id);
    }

    fn pop_open_game_id(&mut self) -> GameId {
        let popped = self.available_games.pop();
        if let Some(open_game_id) = popped {
            open_game_id
        } else {
            panic!(
                "☠️ {} {} {:<8} Out of game IDs!",
                MEGA_DEATH_STRING, MEGA_DEATH_STRING, "UBERFAIL"
            )
        }
    }
}

#[derive(Debug, Clone)]
struct ClientSender {
    pub client_id: ClientId,
    pub events_in: Sender<Events>,
}

#[derive(Debug, Clone)]
pub enum RouterCommand {
    Observe(GameId),
    RequestOpenGame {
        client_id: ClientId,
        events_in: Sender<Events>,
        req_id: ReqId,
    },
    DeleteClient {
        client_id: ClientId,
        game_id: GameId,
    },
    RegisterOpenGame {
        game_id: GameId,
    },
    Reconnect {
        client_id: ClientId,
        game_id: GameId,
        events_in: Sender<Events>,
        req_id: ReqId,
    },
}
