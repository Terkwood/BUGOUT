use std::ops::Add;
use std::str::from_utf8;
use std::time::{Duration, Instant};

use log::{error, info};
use mio_extras::timer::Timeout;

use crossbeam_channel::unbounded;

use ws::util::Token;
use ws::{CloseCode, Error, ErrorKind, Frame, Handler, Handshake, Message, OpCode, Result, Sender};

use crate::backend_commands::*;
use crate::client_commands::*;
use crate::client_events::*;
use crate::idle_status::RequestIdleStatus;
use crate::logging::*;
use crate::model::*;
use crate::router::RouterCommand;
use core_model::*;

const PING: Token = Token(1);
const EXPIRE: Token = Token(2);
const CHANNEL_RECV: Token = Token(3);

const PING_TIMEOUT_MS: u64 = 5_000;
const EXPIRE_TIMEOUT_MS: u64 = 55_000;
const CHANNEL_RECV_TIMEOUT_MS: u64 = 10;

// WebSocket handler
pub struct WsSession {
    pub session_id: SessionId,
    pub ws_out: Sender,
    pub ping_timeout: Option<Timeout>,
    pub expire_timeout: Option<Timeout>,
    pub channel_recv_timeout: Option<Timeout>,
    pub session_commands_in: crossbeam_channel::Sender<BackendCommands>,
    pub events_out: Option<crossbeam_channel::Receiver<ClientEvents>>,
    pub router_commands_in: crossbeam_channel::Sender<RouterCommand>,
    pub req_idle_status_in: crossbeam_channel::Sender<RequestIdleStatus>,
    pub current_game: Option<GameId>,
    pub expire_after: std::time::Instant,
    pub client_id: Option<ClientId>,
}

impl WsSession {
    pub fn new(
        ws_out: ws::Sender,
        session_commands_in: crossbeam_channel::Sender<BackendCommands>,
        router_commands_in: crossbeam_channel::Sender<RouterCommand>,
        req_idle_status_in: crossbeam_channel::Sender<RequestIdleStatus>,
    ) -> WsSession {
        WsSession {
            session_id: SessionId::new(),
            ws_out,
            ping_timeout: None,
            expire_timeout: None,
            channel_recv_timeout: None,
            session_commands_in,
            events_out: None,
            router_commands_in,
            req_idle_status_in,
            current_game: None,
            expire_after: next_expiry(),
            client_id: None,
        }
    }

    fn send_to_backend(
        &self,
        backend_command: BackendCommands,
    ) -> std::result::Result<(), crossbeam::SendError<BackendCommands>> {
        self.session_commands_in.send(backend_command)
    }

    fn notify_router_close(&mut self) {
        if let Err(e) = self.router_commands_in.send(RouterCommand::DeleteSession {
            session_id: self.session_id,
            game_id: self.current_game,
            client_id: self.client_id,
        }) {
            error!(
                "😤 {} ERROR  send router command delete client {}",
                session_code(self),
                e
            )
        }
    }

    /// Observe that someone is still connected to this game
    fn observe_game(&mut self) {
        if let Some(gid) = self.current_game {
            if let Err(_e) = self
                .router_commands_in
                .send(RouterCommand::ObserveGame(gid))
            {
                error!("observe game")
            }
        }
    }

    fn produce_client_heartbeat(&mut self, heartbeat_type: HeartbeatType) {
        if let Some(client_id) = self.client_id {
            if let Err(e) =
                self.send_to_backend(BackendCommands::ClientHeartbeat(ClientHeartbeat {
                    client_id,
                    heartbeat_type,
                }))
            {
                error!("Failed to send client heartbeat via crossbeam {}", e)
            }
        }
    }
}

impl Handler for WsSession {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        info!("🎫 {} OPEN", session_code(self));

        // Router needs to know about this client immediately
        // so that it can handle PROVIDLE, FINDPUBG, JOINPRIV
        let (events_in, events_out) = client_event_channels();

        if let Err(e) = self.router_commands_in.send(RouterCommand::AddSession {
            session_id: self.session_id,
            events_in,
        }) {
            error!(
                "😠 {} {:<8} sending router command to add session {}",
                session_code(self),
                "ERROR",
                e
            )
        }

        // Track the out-channel so we can select! on it
        self.events_out = Some(events_out);

        // schedule a timeout to send a ping every 5 seconds
        self.ws_out.timeout(PING_TIMEOUT_MS, PING)?;
        // schedule a timeout to close the connection if there is no activity for 30 seconds
        self.ws_out.timeout(EXPIRE_TIMEOUT_MS, EXPIRE)?;
        // schedule a timeout to poll for kafka originated
        // events on the crossbeam channel
        self.ws_out.timeout(CHANNEL_RECV_TIMEOUT_MS, CHANNEL_RECV)
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        use move_model::Coord;
        let msg_text = &&msg.into_text()?;
        let deserialized: Result<ClientCommands> = serde_json::from_str(msg_text)
            .map_err(|_err| ws::Error::new(ws::ErrorKind::Internal, "json"));
        match deserialized {
            Ok(ClientCommands::MakeMove(MakeMoveCommand {
                game_id,
                req_id,
                player,
                coord,
            })) => {
                info!(
                    "{}  {} {:<8} {:?} {}",
                    emoji(&player),
                    session_code(self),
                    "MAKEMOVE",
                    player,
                    if let Some(Coord { x, y }) = coord {
                        format!("{{ {:<2}, {:<2} }}", x, y)
                    } else {
                        "PASS".to_string()
                    }
                );

                if let Some(c) = self.current_game {
                    if c == game_id {
                        return self
                            .send_to_backend(BackendCommands::MakeMove(MakeMoveCommand {
                                game_id,
                                req_id,
                                player,
                                coord,
                            }))
                            .map_err(|e| ws::Error::from(Box::new(e)));
                    }
                }
                Ok(self.observe_game())
            }
            Ok(ClientCommands::Beep) => {
                info!("🤖 {} BEEP   ", session_code(self));

                self.produce_client_heartbeat(HeartbeatType::UserInterfaceBeep);
                Ok(self.observe_game())
            }
            Ok(ClientCommands::Reconnect(ReconnectCommand { game_id, req_id })) => {
                if let Some(client_id) = self.client_id {
                    // accept whatever game_id the client shares with us
                    self.current_game = Some(game_id);

                    info!("🔌 {} RECONN", session_code(self));

                    // ..and let the router know we're interested in it,
                    // so that we can receive updates
                    if let Err(e) = self.router_commands_in.send(RouterCommand::Reconnect {
                        client_id,
                        game_id,
                        req_id,
                    }) {
                        error!(
                            "😫 {} {:<8} sending router command to reconnect client {:?}",
                            session_code(self),
                            "ERROR",
                            e
                        )
                    }

                    Ok(self.observe_game())
                } else {
                    complain_no_client_id()
                }
            }
            Ok(ClientCommands::ProvideHistory(ProvideHistoryCommand { game_id, req_id })) => {
                info!("📋 {} PROVHIST", session_code(self));

                if let Err(e) = self
                    .send_to_backend(BackendCommands::ProvideHistory(ProvideHistoryCommand {
                        game_id,
                        req_id,
                    }))
                    .map_err(|e| ws::Error::from(Box::new(e)))
                {
                    error!("ERROR on kafka send provhist {:?}", e)
                }

                Ok(self.observe_game())
            }
            Ok(ClientCommands::FindPublicGame) => {
                // Ignore this request if we already have a game
                // in progress.
                if let (None, Some(client_id)) = (self.current_game, self.client_id) {
                    info!("🤝 {} FINDPUBG", session_code(self));

                    if let Err(e) = self.send_to_backend(BackendCommands::FindPublicGame(
                        FindPublicGameBackendCommand {
                            client_id,
                            session_id: self.session_id,
                        },
                    )) {
                        error!(
                            "😠 {} {:<8} kafka sending find public game command {}",
                            session_code(self),
                            "ERROR",
                            e
                        )
                    }
                }
                Ok(self.observe_game())
            }
            Ok(ClientCommands::CreatePrivateGame(cp)) => {
                // Ignore this request if we already have a game
                // in progress.
                if let (None, Some(client_id)) = (self.current_game, self.client_id) {
                    info!("🔒 {} CRETPRIV", session_code(self));

                    let board_size = cp.board_size.unwrap_or(crate::FULL_BOARD_SIZE);

                    if let Err(e) = self
                        .send_to_backend(BackendCommands::CreateGame(CreateGameBackendCommand {
                            client_id,
                            visibility: Visibility::Private,
                            session_id: self.session_id,
                            board_size,
                        }))
                        .map_err(|e| ws::Error::from(Box::new(e)))
                    {
                        error!("ERROR on kafka send join private game {:?}", e)
                    }
                }
                Ok(self.observe_game())
            }
            Ok(ClientCommands::JoinPrivateGame(JoinPrivateGameClientCommand { game_id })) => {
                info!("🔑 {} JOINPRIV", session_code(self));

                // Ignore this request if we already have a game
                // in progress.
                if self.current_game.is_none() {
                    if let (Some(game_id), Some(client_id)) = (game_id.decode(), self.client_id) {
                        if let Err(e) = self
                            .send_to_backend(BackendCommands::JoinPrivateGame(
                                JoinPrivateGameBackendCommand {
                                    game_id,
                                    client_id,
                                    session_id: self.session_id,
                                },
                            ))
                            .map_err(|e| ws::Error::from(Box::new(e)))
                        {
                            error!("ERROR on kafka send join private game {:?}", e)
                        }
                    }
                } else {
                    error!("🏴‍☠️ FAILED TO DECODE PRIVATE GAME ID 🏴‍☠️")
                }
                Ok(self.observe_game())
            }
            Ok(ClientCommands::ChooseColorPref(ChooseColorPrefClientCommand { color_pref })) => {
                if let Some(client_id) = self.client_id {
                    info!("🗳  {} CHSCLRPF", session_code(self));

                    self.send_to_backend(BackendCommands::ChooseColorPref(
                        ChooseColorPrefBackendCommand {
                            client_id,
                            color_pref,
                            session_id: self.session_id,
                        },
                    ))
                    .map_err(|e| ws::Error::from(Box::new(e)))
                } else {
                    complain_no_client_id()
                }
            }
            Ok(ClientCommands::ProvideIdleStatus) => {
                if let Some(client_id) = self.client_id {
                    info!("🏕  {} PROVIDLE", session_code(self));

                    // Request the idle status
                    self.req_idle_status_in
                        .send(RequestIdleStatus(client_id))
                        .map_err(|e| ws::Error::from(Box::new(e)))
                } else {
                    complain_no_client_id()
                }
            }
            Ok(ClientCommands::Identify(id)) => {
                self.client_id = Some(id.client_id);
                info!("🆔 {} IDENTIFY", session_code(self));

                self.router_commands_in
                    .send(RouterCommand::IdentifyClient {
                        session_id: self.session_id,
                        client_id: id.client_id,
                    })
                    .map_err(|e| ws::Error::from(Box::new(e)))
                    .and_then(|_a| {
                        self.ws_out.send(
                            serde_json::to_string(&ClientEvents::IdentityAcknowledged(id)).unwrap(),
                        )
                    })
            }
            Ok(ClientCommands::QuitGame) => {
                if let (Some(client_id), Some(game_id)) = (self.client_id, self.current_game) {
                    info!("🏳️  {} {:<8}", session_code(self), "QUITGAME");

                    self.current_game = None;

                    if let Err(e) = self.router_commands_in.send(RouterCommand::QuitGame {
                        session_id: self.session_id,
                        game_id,
                    }) {
                        error!("ERROR SENDING ROUTER QUIT COMMAND: {}", e);
                    }

                    self.send_to_backend(BackendCommands::QuitGame(QuitGameCommand {
                        client_id,
                        game_id,
                    }))
                    .map_err(|e| ws::Error::from(Box::new(e)))
                } else {
                    error!("Can't quit without client ID + game ID");
                    Ok(())
                }
            }
            Ok(ClientCommands::AttachBot(AttachBotClientCommand {
                player: lp,
                board_size,
            })) => {
                use core_model::GameId;
                info!("📌 {} ATACHBOT", session_code(self));

                use move_model::Player;
                let player = match lp {
                    Player::BLACK => Player::BLACK,
                    _ => Player::WHITE,
                };
                let game_id = uuid::Uuid::new_v4();
                if let Err(e) = self.router_commands_in.send(RouterCommand::RouteGame {
                    session_id: self.session_id,
                    game_id,
                }) {
                    error!("failed to send RouteGame command {:?}", e)
                }

                Ok({
                    let payload = BackendCommands::AttachBot(micro_model_bot::gateway::AttachBot {
                        game_id: micro_model_moves::GameId(game_id),
                        player,
                        board_size,
                    });

                    if let Err(e) = self.session_commands_in.send(payload) {
                        error!("could not set up bot backend {:?}", e)
                    }

                    self.current_game = Some(game_id);
                })
            }
            Ok(ClientCommands::ReqSync(ReqSyncClientCommand {
                req_id,
                turn,
                player_up,
                last_move,
            })) => {
                if let Some(game_id) = self.current_game {
                    info!("📥 {} {:<8}", session_code(self), "REQSYNC");
                    if let Err(e) = self
                        .send_to_backend(BackendCommands::ReqSync(ReqSyncBackendCommand {
                            req_id,
                            session_id: self.session_id,
                            turn,
                            player_up,
                            last_move,
                            game_id,
                        }))
                        .map_err(|e| ws::Error::from(Box::new(e)))
                    {
                        error!("💥 Req sync {:?}", e)
                    }
                }

                Ok(())
            }
            Err(_err) => {
                error!(
                    "💥 {} {:<8} message deserialization {}",
                    session_code(self),
                    "ERROR",
                    msg_text
                );
                Ok(())
            }
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!(
            "🚪 {} {:<8} {:?} {}",
            session_code(self),
            "CLOSE",
            code,
            reason
        );

        self.notify_router_close();

        // Clean up timeouts when connections close
        if let Some(t) = self.ping_timeout.take() {
            self.ws_out.cancel(t).unwrap();
        }
        if let Some(t) = self.expire_timeout.take() {
            self.ws_out.cancel(t).unwrap();
        }
        if let Some(t) = self.channel_recv_timeout.take() {
            self.ws_out.cancel(t).unwrap();
        }
        // This is ultimately consumed by game lobby
        // and helps clean up abandoned games
        if let Err(e) =
            self.send_to_backend(BackendCommands::SessionDisconnected(SessionDisconnected {
                session_id: self.session_id,
            }))
        {
            error!("Couldn't send client disconnect to kafka {}", e)
        }
    }

    fn on_error(&mut self, err: Error) {
        // Log any error
        error!("🔥 {} {:<8} {:?}", session_code(self), "ERROR", err)
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        use move_model::Player;
        match event {
            // PING timeout has occured, send a ping and reschedule
            PING => {
                self.ws_out
                    .ping(time::precise_time_ns().to_string().into())?;
                self.ping_timeout.take();
                self.ws_out.timeout(PING_TIMEOUT_MS, PING)
            }
            EXPIRE => {
                if let Some(dur) = self.expire_after.checked_duration_since(Instant::now()) {
                    // EXPIRE timeout has occured.
                    // this means that the connection is inactive
                    // so we close it
                    if dur.as_millis() >= EXPIRE_TIMEOUT_MS.into() {
                        info!(
                            "⌛️ {} {:<8} Closing connection",
                            session_code(self),
                            "EXPIRE"
                        );
                        self.notify_router_close();
                        return self.ws_out.close(CloseCode::Away);
                    }
                }

                Ok(())
            }
            CHANNEL_RECV => {
                if let Some(eo) = &self.events_out {
                    while let Ok(event) = eo.try_recv() {
                        match &event {
                            ClientEvents::GameReady(GameReadyClientEvent {
                                game_id,
                                event_id: _,
                                board_size: _,
                            }) => {
                                self.current_game = Some(game_id.clone());
                                info!("🎳 {} {:<8}", session_code(self), "GAMEREDY");
                            }
                            ClientEvents::WaitForOpponent(WaitForOpponentClientEvent {
                                game_id,
                                event_id: _,
                                visibility: _,
                                link: _,
                            }) => {
                                self.current_game = Some(game_id.clone());
                                info!("⏳ {} {:<8}", session_code(self), "WAITOPPO");
                            }
                            ClientEvents::YourColor(YourColorEvent {
                                game_id: _,
                                your_color,
                            }) if your_color == &Player::BLACK => {
                                info!("⚫️ {} {:<8} Black", session_code(self), "YOURCOLR")
                            }
                            ClientEvents::YourColor(YourColorEvent {
                                game_id: _,
                                your_color,
                            }) if your_color == &Player::WHITE => {
                                info!("⚪️ {} {:<8} White", session_code(self), "YOURCOLR")
                            }
                            ClientEvents::OpponentQuit => {
                                self.current_game = None;
                            }
                            ClientEvents::MoveMade(m) => info!(
                                "🆗 {} {:<8} {} {:?}",
                                session_code(self),
                                "MOVEMADE",
                                m.player,
                                m.coord
                            ),
                            ClientEvents::SyncReply(_) => {
                                info!("📤 {} {:<8}", session_code(self), "SYNCRPLY")
                            }
                            _ => (),
                        }

                        self.ws_out.send(serde_json::to_string(&event).unwrap())?
                    }
                }
                self.channel_recv_timeout.take();
                self.ws_out.timeout(CHANNEL_RECV_TIMEOUT_MS, CHANNEL_RECV)
            }
            // No other timeouts are possible
            _ => Err(Error::new(
                ErrorKind::Internal,
                "Invalid timeout token encountered!",
            )),
        }
    }

    fn on_new_timeout(&mut self, event: Token, timeout: Timeout) -> Result<()> {
        // Cancel the old timeout and replace.
        if event == EXPIRE {
            if let Some(t) = self.expire_timeout.take() {
                self.ws_out.cancel(t)?
            }
            self.expire_timeout = Some(timeout);
            self.expire_after = next_expiry()
        } else if event == PING {
            // This ensures there is only one timeout at a time
            if let Some(t) = self.ping_timeout.take() {
                self.ws_out.cancel(t)?
            }
            self.ping_timeout = Some(timeout)
        } else if event == CHANNEL_RECV {
            if let Some(t) = self.channel_recv_timeout.take() {
                self.ws_out.cancel(t)?
            }
            self.channel_recv_timeout = Some(timeout)
        }

        Ok(())
    }

    fn on_frame(&mut self, frame: Frame) -> Result<Option<Frame>> {
        // If the frame is a pong, print the round-trip time.
        // The pong should contain data from out ping, but it isn't guaranteed to.
        if frame.opcode() == OpCode::Pong {
            if let Ok(pong) = from_utf8(frame.payload())?.parse::<u64>() {
                self.observe_game();
                self.produce_client_heartbeat(HeartbeatType::WebSocketPong);

                let now = time::precise_time_ns();
                info!(
                    "🏓 {} {:<8} {:.0}ms",
                    session_code(self),
                    "PINGPONG",
                    (now - pong) as f64 / 1_000_000f64
                );
            } else {
                error!("😐 {} {:<8} gOnE wRoNg", session_code(self), "PINGPONG");
            }
        }

        // Some activity has occured, so reset the expiration
        self.ws_out.timeout(EXPIRE_TIMEOUT_MS, EXPIRE)?;

        // Run default frame validation
        DefaultHandler.on_frame(frame)
    }
}

// For accessing the default handler implementation
struct DefaultHandler;

impl Handler for DefaultHandler {}

fn next_expiry() -> Instant {
    Instant::now().add(Duration::from_millis(EXPIRE_TIMEOUT_MS))
}

fn client_event_channels() -> (
    crossbeam_channel::Sender<ClientEvents>,
    crossbeam_channel::Receiver<ClientEvents>,
) {
    unbounded()
}

fn complain_no_client_id() -> Result<()> {
    Ok(error!("❌ UNEXPECTED: NO CLIENT ID DEFINED ❌"))
}
