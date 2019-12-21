use std::ops::Add;
use std::str::from_utf8;
use std::time::{Duration, Instant};

use mio_extras::timer::Timeout;

use crossbeam_channel::unbounded;

use ws::util::Token;
use ws::{CloseCode, Error, ErrorKind, Frame, Handler, Handshake, Message, OpCode, Result, Sender};

use crate::client_commands::*;
use crate::client_events::*;
use crate::idle_status::RequestIdleStatus;
use crate::kafka_commands::*;
use crate::logging::*;
use crate::model::*;
use crate::router::RouterCommand;

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
    pub kafka_commands_in: crossbeam_channel::Sender<KafkaCommands>,
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
        kafka_commands_in: crossbeam_channel::Sender<KafkaCommands>,
        router_commands_in: crossbeam_channel::Sender<RouterCommand>,
        req_idle_status_in: crossbeam_channel::Sender<RequestIdleStatus>,
    ) -> WsSession {
        WsSession {
            session_id: uuid::Uuid::new_v4(),
            ws_out,
            ping_timeout: None,
            expire_timeout: None,
            channel_recv_timeout: None,
            kafka_commands_in,
            events_out: None,
            router_commands_in,
            req_idle_status_in,
            current_game: None,
            expire_after: next_expiry(),
            client_id: None,
        }
    }

    fn notify_router_close(&mut self) {
        if let Err(e) = self.router_commands_in.send(RouterCommand::DeleteSession {
            session_id: self.session_id,
            game_id: self.current_game,
            client_id: self.client_id,
        }) {
            println!(
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
                println!("eeeeerrrrrr")
            }
        }
    }

    fn produce_client_heartbeat(&mut self, heartbeat_type: HeartbeatType) {
        if let Some(client_id) = self.client_id {
            if let Err(e) =
                self.kafka_commands_in
                    .send(KafkaCommands::ClientHeartbeat(ClientHeartbeat {
                        client_id,
                        heartbeat_type,
                    }))
            {
                println!("Failed to send client heartbeat via crossbeam {}", e)
            }
        }
    }
}

impl Handler for WsSession {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        println!("🎫 {} OPEN", session_code(self));

        // Router needs to know about this client immediately
        // so that it can handle PROVIDLE, FINDPUBG, JOINPRIV
        let (events_in, events_out) = client_event_channels();

        if let Err(e) = self.router_commands_in.send(RouterCommand::AddSession {
            session_id: self.session_id,
            events_in,
        }) {
            println!(
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
                println!(
                    "{}  {} {:<8} {:?} {}",
                    emoji(&player),
                    session_code(self),
                    "MOVE",
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
                            .kafka_commands_in
                            .send(KafkaCommands::MakeMove(MakeMoveCommand {
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
                println!("🤖 {} BEEP   ", session_code(self));

                self.produce_client_heartbeat(HeartbeatType::UserInterfaceBeep);
                Ok(self.observe_game())
            }
            Ok(ClientCommands::Reconnect(ReconnectCommand { game_id, req_id })) => {
                if let Some(client_id) = self.client_id {
                    // accept whatever game_id the client shares with us
                    self.current_game = Some(game_id);

                    println!("🔌 {} RECONN", session_code(self));

                    // ..and let the router know we're interested in it,
                    // so that we can receive updates
                    if let Err(e) = self.router_commands_in.send(RouterCommand::Reconnect {
                        client_id,
                        game_id,
                        req_id,
                    }) {
                        println!(
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
                println!("📋 {} PROVHIST", session_code(self));

                if let Err(e) = self
                    .kafka_commands_in
                    .send(KafkaCommands::ProvideHistory(ProvideHistoryCommand {
                        game_id,
                        req_id,
                    }))
                    .map_err(|e| ws::Error::from(Box::new(e)))
                {
                    println!("ERROR on kafka send provhist {:?}", e)
                }

                Ok(self.observe_game())
            }
            Ok(ClientCommands::FindPublicGame) => {
                // Ignore this request if we already have a game
                // in progress.
                if let (None, Some(client_id)) = (self.current_game, self.client_id) {
                    println!("🤝 {} FINDPUBG", session_code(self));

                    // ..and let the router know we're interested in it,
                    // so that we can receive updates
                    if let Err(e) = self.kafka_commands_in.send(KafkaCommands::FindPublicGame(
                        FindPublicGameKafkaCommand { client_id },
                    )) {
                        println!(
                            "😠 {} {:<8} kafka sending find public game command {}",
                            session_code(self),
                            "ERROR",
                            e
                        )
                    }
                }
                Ok(self.observe_game())
            }
            Ok(ClientCommands::CreatePrivateGame) => {
                // Ignore this request if we already have a game
                // in progress.
                if let (None, Some(client_id)) = (self.current_game, self.client_id) {
                    println!("🔒 {} CRETPRIV", session_code(self));

                    // ..and let the router know we're interested in it,
                    // so that we can receive updates
                    if let Err(e) = self
                        .kafka_commands_in
                        .send(KafkaCommands::CreateGame(CreateGameKafkaCommand {
                            client_id,
                            visibility: Visibility::Private,
                        }))
                        .map_err(|e| ws::Error::from(Box::new(e)))
                    {
                        println!("ERROR on kafka send join private game {:?}", e)
                    }
                }
                Ok(self.observe_game())
            }
            Ok(ClientCommands::JoinPrivateGame(JoinPrivateGameClientCommand { game_id })) => {
                println!("🔑 {} JOINPRIV", session_code(self));

                // Ignore this request if we already have a game
                // in progress.
                if self.current_game.is_none() {
                    if let (Some(game_id), Some(client_id)) = (game_id.decode(), self.client_id) {
                        if let Err(e) = self
                            .kafka_commands_in
                            .send(KafkaCommands::JoinPrivateGame(
                                JoinPrivateGameKafkaCommand { game_id, client_id },
                            ))
                            .map_err(|e| ws::Error::from(Box::new(e)))
                        {
                            println!("ERROR on kafka send join private game {:?}", e)
                        }
                    }
                } else {
                    println!("🏴‍☠️ FAILED TO DECODE PRIVATE GAME ID 🏴‍☠️")
                }
                Ok(self.observe_game())
            }
            Ok(ClientCommands::ChooseColorPref(ChooseColorPrefClientCommand { color_pref })) => {
                if let Some(client_id) = self.client_id {
                    println!("🗳  {} CHSCLRPF", session_code(self));

                    self.kafka_commands_in
                        .send(KafkaCommands::ChooseColorPref(
                            ChooseColorPrefKafkaCommand {
                                client_id,
                                color_pref,
                            },
                        ))
                        .map_err(|e| ws::Error::from(Box::new(e)))
                } else {
                    complain_no_client_id()
                }
            }
            Ok(ClientCommands::ProvideIdleStatus) => {
                if let Some(client_id) = self.client_id {
                    println!("🏕  {} PROVIDLE", session_code(self));

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
                println!("🆔 {} IDENTIFY", session_code(self));

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
            Err(_err) => {
                println!(
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
        println!(
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

        if let Some(client_id) = self.client_id {
            // This is ultimately consumed by game lobby
            // and helps clean up abandoned games
            if let Err(e) = self
                .kafka_commands_in
                .send(KafkaCommands::ClientDisconnected(ClientDisconnected {
                    client_id,
                }))
            {
                println!("Couldn't send client disconnect to kafka {}", e)
            }
        }
    }

    fn on_error(&mut self, err: Error) {
        // Log any error
        println!("🔥 {} {:<8} {:?}", session_code(self), "ERROR", err)
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
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
                        println!(
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
                        match event {
                            ClientEvents::GameReady(GameReadyClientEvent {
                                game_id,
                                event_id: _,
                            }) => {
                                self.current_game = Some(game_id);
                                println!("🎳 {} {:<8}", session_code(self), "GAMEREDY");
                            }
                            ClientEvents::WaitForOpponent(WaitForOpponentClientEvent {
                                game_id,
                                event_id: _,
                                visibility: _,
                                link: _,
                            }) => {
                                self.current_game = Some(game_id);
                                println!("⏳ {} {:<8}", session_code(self), "WAITOPPO");
                            }
                            ClientEvents::YourColor(YourColorEvent {
                                game_id: _,
                                your_color,
                            }) if your_color == Player::BLACK => {
                                println!("🏴 {} {:<8} Black", session_code(self), "YOURCOLR")
                            }
                            ClientEvents::YourColor(YourColorEvent {
                                game_id: _,
                                your_color,
                            }) if your_color == Player::WHITE => {
                                println!("🏳  {} {:<8} White", session_code(self), "YOURCOLR")
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
                println!(
                    "🏓 {} {:<8} {:.0}ms",
                    session_code(self),
                    "PINGPONG",
                    (now - pong) as f64 / 1_000_000f64
                );
            } else {
                println!("😐 {} {:<8} gOnE wRoNg", session_code(self), "PINGPONG");
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
    println!("NO CLIENT ID DEFINED");
    Ok(())
}
