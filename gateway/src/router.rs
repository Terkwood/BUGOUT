use crossbeam_channel::{select, Receiver, Sender};
use log::{error, info, warn};
use std::collections::HashMap;
use std::ops::Add;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::backend::events::BackendEvents;
use crate::client_events::{ClientEvents, YourColorEvent};
use crate::idle_status::IdleStatusResponse;
use crate::model::*;
use crate::{short_uuid, EMPTY_SHORT_UUID};

const GAME_CLIENT_CLEANUP_PERIOD_MS: u64 = 10_000;

/// Keeps track of clients interested in various games
/// Each client has an associated crossbeam Sender for BUGOUT events
struct Router {
    pub game_sessions: HashMap<GameId, GameSessions>,
    pub last_cleanup: Instant,
    pub sessions: HashMap<SessionId, Sender<ClientEvents>>,
    pub client_sessions: HashMap<ClientId, SessionSender>,
}

impl Router {
    pub fn new() -> Router {
        Router {
            game_sessions: HashMap::new(),
            last_cleanup: Instant::now(),
            sessions: HashMap::new(),
            client_sessions: HashMap::new(),
        }
    }

    pub fn forward_by_session_id(&self, session_id: SessionId, ev: ClientEvents) {
        if let Some(events_in) = self.sessions.get(&session_id) {
            if let Err(e) = events_in.send(ev.clone()) {
                error!(
                    "😗 {} {:<8} {:<8} forwarding event by session ID {}",
                    "", "", "ERROR", e
                )
            }
        }
    }

    pub fn forward_by_client_id(&self, client_id: ClientId, ev: ClientEvents) {
        if let Some(session_sender) = self.client_sessions.get(&client_id) {
            if let Err(e) = session_sender.events_in.send(ev.clone()) {
                error!(
                    "😗 {} {:<8} {:<8} forwarding event by client ID {}",
                    short_uuid(client_id),
                    "",
                    "ERROR",
                    e
                )
            }
        } else {
            error!("Could not forward to client ID, perhaps it was already cleaned up ?")
        }
    }

    pub fn forward_by_game_id(&self, ev: ClientEvents) {
        if let Some(gid) = &ev.game_id() {
            if let Some(GameSessions {
                sessions,
                playerup: _,
                modified_at: _,
            }) = self.game_sessions.get(gid)
            {
                for s in sessions {
                    if let Err(err) = s.events_in.send(ev.clone()) {
                        error!(
                            "😑 {} {} {:<8} forwarding event by game ID {}",
                            &crate::EMPTY_SHORT_UUID,
                            short_uuid(*gid),
                            "ERROR",
                            err
                        )
                    }
                }
            } else {
                warn!("No match {:?}", ev);
            }
        }
    }

    pub fn playerup(&self, game_id: GameId) -> Player {
        self.game_sessions
            .get(&game_id)
            .map(|game_client| &game_client.playerup)
            .unwrap_or(&Player::BLACK)
            .clone()
    }

    /// Note that the game is active, so that
    /// we don't purge it prematurely in cleanup_game_clients()
    pub fn observe_game(&mut self, game_id: GameId) {
        self.game_sessions
            .get_mut(&game_id)
            .map(|gs| gs.observe_game());
    }

    pub fn set_playerup(&mut self, game_id: GameId, player: Player) {
        let c = player.clone();
        let default = GameSessions {
            sessions: vec![],
            playerup: c,
            modified_at: Instant::now(),
        };
        let gs = self.game_sessions.get_mut(&game_id);
        match gs {
            None => {
                self.game_sessions.insert(game_id, default);
            }
            Some(gs) => {
                gs.playerup = player;
                gs.observe_game();
            }
        }
    }

    pub fn route_new_game(&mut self, session_id: SessionId, game_id: GameId) {
        let x = self.sessions.get(&session_id);
        if let Some(events_in) = x {
            let newbie = SessionSender {
                session_id,
                events_in: events_in.clone(),
            };

            match self.game_sessions.get_mut(&game_id) {
                Some(gs) => gs.add_session(newbie),
                None => {
                    let with_newbie = GameSessions::new(newbie);
                    self.game_sessions.insert(game_id, with_newbie);
                }
            }
        }
    }
    fn reconnect(
        &mut self,
        session_id: SessionId,
        game_id: GameId,
        events_in: Sender<ClientEvents>,
    ) {
        let cs = SessionSender {
            session_id,
            events_in,
        };

        match self.game_sessions.get_mut(&game_id) {
            Some(gs) => gs.add_session(cs),
            None => {
                self.game_sessions.insert(game_id, GameSessions::new(cs));
            }
        }
    }

    fn find_dead_game_sessions(&mut self) -> Vec<GameId> {
        let mut to_delete = vec![];

        for (game_id, game_sessions) in self.game_sessions.iter() {
            if game_sessions.sessions.len() == 0 {
                let since = Instant::now().checked_duration_since(
                    game_sessions
                        .modified_at
                        .add(Duration::from_millis(GAME_CLIENT_CLEANUP_PERIOD_MS)),
                );

                if let Some(dur) = since {
                    if dur.as_millis() > GAME_CLIENT_CLEANUP_PERIOD_MS.into() {
                        to_delete.push(*game_id);
                    }
                }
            }
        }

        to_delete
    }

    fn cleanup_game_sessions(&mut self) {
        let since = Instant::now().checked_duration_since(self.last_cleanup);
        if let Some(dur) = since {
            if dur.as_millis() > GAME_CLIENT_CLEANUP_PERIOD_MS.into() {
                let to_delete = self.find_dead_game_sessions();
                let mut count = 0;
                for game_id in to_delete {
                    if let Some(g) = self.game_sessions.get(&game_id) {
                        for c in &g.sessions {
                            self.sessions.remove_entry(&c.session_id);
                        }
                    }
                    self.game_sessions.remove_entry(&game_id);

                    count += 1;
                }

                self.last_cleanup = Instant::now();
                if count > 0 {
                    info!(
                        "🗑 {} {}  {:<8} {:<4} game records",
                        EMPTY_SHORT_UUID, EMPTY_SHORT_UUID, "CLEANUP", count
                    )
                }
            }
        }
    }

    pub fn delete_session(
        &mut self,
        session_id: SessionId,
        game_id: Option<GameId>,
        client_id: Option<ClientId>,
    ) {
        if let Some(gid) = game_id {
            let mut sess_keys = self.sessions.keys();
            if let Some(game_session) = self.game_sessions.get_mut(&gid) {
                game_session.sessions.retain(|c| {
                    c.session_id != session_id && sess_keys.any(|k| *k == c.session_id)
                    // only keep those tied to active sessions
                });
            }
        }

        if let Some(cid) = client_id {
            self.client_sessions.remove(&cid);
        }

        self.sessions.remove(&session_id);

        self.cleanup_game_sessions();
    }
}

/// start the select! loop responsible for sending
/// kafka messages to relevant websocket clients.
/// it must respond to requests to add and drop listeners
pub fn start(
    router_commands_out: Receiver<RouterCommand>,
    backend_events_out: Receiver<BackendEvents>,
    idle_resp_out: Receiver<IdleStatusResponse>,
) {
    thread::spawn(move || {
        let mut router = Router::new();
        loop {
            select! {
            recv(router_commands_out) -> command =>
                match command {
                    Ok(RouterCommand::ObserveGame(game_id)) => router.observe_game(game_id),
                    // A create private game request, or a find public
                    // game request, will result in us tracking a
                    // client_id -> event channel mapping
                    // We'll use this to send messages back to the browser,
                    // later
                    Ok(RouterCommand::AddSession { session_id, events_in }) => {
                        router.sessions.insert(session_id, events_in);
                    },
                    Ok(RouterCommand::DeleteSession{session_id, game_id, client_id}) =>
                        router.delete_session(session_id, game_id, client_id),
                    Ok(RouterCommand::Reconnect{client_id, game_id, req_id }) => {
                        if let Some(session_sender) = router.client_sessions.get(&client_id) {
                            let event_clone = session_sender.events_in.clone();
                            router.reconnect(client_id, game_id, event_clone.clone());
                            if let Err(err) = event_clone.send(ClientEvents::Reconnected(ReconnectedEvent{game_id, reply_to: req_id, event_id: Uuid::new_v4(), player_up: router.playerup(game_id)})) {
                                error!(
                                    "😦 {} {} {:<8} could not send reconnect reply {}",
                                    short_uuid(client_id),
                                    short_uuid(game_id),
                                    "ERROR", err)
                            }
                        }
                    },
                    Ok(RouterCommand::IdentifyClient {
                        session_id, client_id,
                    }) => if let Some(events_in) = router.sessions.get(&session_id) {
                        router.client_sessions.insert(client_id, SessionSender{session_id, events_in:events_in.clone()});
                    },
                    Ok(RouterCommand::QuitGame { session_id, game_id }) => {
                        if let Some(GameSessions { sessions, playerup: _, modified_at: _}) = router.game_sessions.get(&game_id) {
                            for game_session in sessions {
                                if game_session.session_id != session_id {
                                    if let Err(e) = game_session.events_in.send(ClientEvents::OpponentQuit) {
                                        error!("failed to pass along Opponent Quit : {}", e)
                                    }
                                }
                            }
                        }

                        router.game_sessions.remove(&game_id);
                    },
                    Ok(RouterCommand::RouteGame { session_id, game_id }) =>
                        router.route_new_game(session_id, game_id),
                    Err(e) => panic!("Unable to receive command via router channel: {:?}", e),
                },
            recv(backend_events_out) -> event =>
                match event {
                    Ok(BackendEvents::MoveMade(m)) => {
                        let u = m.clone();
                        router.set_playerup(u.game_id, u.player.other());
                        router.forward_by_game_id(BackendEvents::MoveMade(m).to_client_event())
                    }
                    Ok(BackendEvents::GameReady(g)) => {
                        router.route_new_game(g.sessions.first, g.game_id);
                        router.route_new_game(g.sessions.second, g.game_id);
                        router.forward_by_game_id(BackendEvents::GameReady(g).to_client_event());
                    }
                    Ok(BackendEvents::PrivateGameRejected(p)) => {
                        // there's no game ID associated with
                        // this game, yet, so we need to
                        // forward via client ID
                        router.forward_by_client_id(p.client_id, BackendEvents::PrivateGameRejected(p).to_client_event())
                    }
                    Ok(BackendEvents::WaitForOpponent(w)) => {
                        router.forward_by_session_id(w.session_id, BackendEvents::WaitForOpponent(w).to_client_event())
                    }
                    Ok(BackendEvents::ColorsChosen(ColorsChosenEvent { game_id, black, white })) => {
                        // We want to forward by session ID
                        // so that we don't send TWO yourcolor events
                        // to each client
                        router.forward_by_client_id(black, ClientEvents::YourColor (YourColorEvent{ game_id, your_color: Player::BLACK }));
                        router.forward_by_client_id(white, ClientEvents::YourColor(YourColorEvent{ game_id, your_color: Player::WHITE }));
                    },
                    Ok(BackendEvents::SyncReply(sr)) => {
                        let sess = sr.session_id.clone();
                        let e = BackendEvents::SyncReply(sr);
                        router.observe_game(e.game_id());
                        router.forward_by_session_id(sess, e.to_client_event());
                    },
                    Ok(e) => {
                        router.observe_game(e.game_id());

                        router.forward_by_game_id(e.to_client_event())
                    },
                    Err(e) =>
                        panic!("Unable to receive kafka event via router channel: {:?}", e),
                },
            recv(idle_resp_out) -> idle_status_response => if let Ok(IdleStatusResponse(client_id, status)) = idle_status_response {
                router.forward_by_client_id(client_id, ClientEvents::IdleStatusProvided(status))
            } else {
                error!("router error reading idle response")
            }}
        }
    });
}

/// Maps games to sessions (up to two players for each game) and their event inputs
#[derive(Debug)]
struct GameSessions {
    pub sessions: Vec<SessionSender>,
    pub playerup: Player,
    pub modified_at: Instant,
}

impl GameSessions {
    pub fn new(session: SessionSender) -> GameSessions {
        GameSessions {
            sessions: vec![session],
            playerup: Player::BLACK,
            modified_at: Instant::now(),
        }
    }

    pub fn add_session(&mut self, session: SessionSender) {
        self.sessions.push(session);
        self.modified_at = Instant::now()
    }

    /// Observe that this game still has an open connection somewhere
    pub fn observe_game(&mut self) {
        self.modified_at = Instant::now()
    }
}

#[derive(Debug, Clone)]
struct SessionSender {
    pub session_id: SessionId,
    pub events_in: Sender<ClientEvents>,
}

#[derive(Debug, Clone)]
pub enum RouterCommand {
    ObserveGame(GameId),
    DeleteSession {
        session_id: SessionId,
        game_id: Option<GameId>,
        client_id: Option<ClientId>,
    },
    Reconnect {
        client_id: ClientId,
        game_id: GameId,
        req_id: ReqId,
    },
    AddSession {
        session_id: SessionId,
        events_in: Sender<ClientEvents>,
    },
    IdentifyClient {
        session_id: SessionId,
        client_id: ClientId,
    },
    QuitGame {
        session_id: SessionId,
        game_id: GameId,
    },
    RouteGame {
        session_id: SessionId,
        game_id: GameId,
    },
}
