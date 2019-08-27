use std::ops::Add;
use std::str::from_utf8;
use std::time::{Duration, Instant};

use mio_extras::timer::Timeout;

use crossbeam_channel::unbounded;

use ws::util::Token;
use ws::{CloseCode, Error, ErrorKind, Frame, Handler, Handshake, Message, OpCode, Result, Sender};

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
    pub client_id: ClientId,
    pub ws_out: Sender,
    pub ping_timeout: Option<Timeout>,
    pub expire_timeout: Option<Timeout>,
    pub channel_recv_timeout: Option<Timeout>,
    pub kafka_commands_in: crossbeam_channel::Sender<KafkaCommands>,
    pub events_out: Option<crossbeam_channel::Receiver<ClientEvents>>,
    pub router_commands_in: crossbeam_channel::Sender<RouterCommand>,
    pub current_game: Option<GameId>,
    pub expire_after: std::time::Instant,
}

impl WsSession {
    pub fn new(
        client_id: ClientId,
        ws_out: ws::Sender,
        kafka_commands_in: crossbeam_channel::Sender<KafkaCommands>,
        router_commands_in: crossbeam_channel::Sender<RouterCommand>,
    ) -> WsSession {
        WsSession {
            client_id,
            ws_out,
            ping_timeout: None,
            expire_timeout: None,
            channel_recv_timeout: None,
            kafka_commands_in,
            events_out: None,
            router_commands_in,
            current_game: None,
            expire_after: next_expiry(),
        }
    }

    fn notify_router_close(&mut self) {
        if let Some(game_id) = self.current_game {
            if let Err(e) = self.router_commands_in.send(RouterCommand::DeleteClient {
                client_id: self.client_id,
                game_id,
            }) {
                println!(
                    "😤 {} ERROR  send router command delete client {}",
                    session_code(self),
                    e
                )
            }
        }
    }

    fn observe(&mut self) {
        if let Some(gid) = self.current_game {
            if let Err(_e) = self.router_commands_in.send(RouterCommand::Observe(gid)) {
                println!("eeeeerrrrrr")
            }
        }
    }
}

impl Handler for WsSession {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        println!("🎫 {} OPEN", session_code(self));

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
                Ok(self.observe())
            }
            Ok(ClientCommands::Beep) => {
                println!("🤖 {} BEEP   ", session_code(self));

                Ok(self.observe())
            }
            Ok(ClientCommands::RequestOpenGame(req)) => {
                // Ignore this request if we already have a game
                // in progress.
                if self.current_game.is_none() {
                    let (events_in, events_out) = client_event_channels();

                    // ..and let the router know we're interested in it,
                    // so that we can receive updates
                    if let Err(e) = self
                        .router_commands_in
                        .send(RouterCommand::RequestOpenGame {
                            client_id: self.client_id,
                            events_in,
                            req_id: req.req_id,
                        }) {
                        println!(
                            "😠 {} {:<8} sending router command to add client {}",
                            session_code(self),
                            "ERROR",
                            e
                        )
                    }

                    //.. and track the out-channel so we can select! on it
                    self.events_out = Some(events_out);
                }
                Ok(self.observe())
            }
            Ok(ClientCommands::Reconnect(ReconnectCommand { game_id, req_id })) => {
                // accept whatever game_id the client shares with us
                self.current_game = Some(game_id);

                println!("🔌 {} RECONN", session_code(self));
                let (events_in, events_out) = client_event_channels();

                // ..and let the router know we're interested in it,
                // so that we can receive updates
                if let Err(e) = self.router_commands_in.send(RouterCommand::Reconnect {
                    client_id: self.client_id,
                    game_id,
                    events_in,
                    req_id,
                }) {
                    println!(
                        "😫 {} {:<8} sending router command to reconnect client {:?}",
                        session_code(self),
                        "ERROR",
                        e
                    )
                }

                //.. and track the out-channel so we can select! on it
                self.events_out = Some(events_out);

                Ok(self.observe())
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

                Ok(self.observe())
            }
            Ok(ClientCommands::JoinPrivateGame(JoinPrivateGameClientCommand { game_id })) => {
                // Ignore this request if we already have a game
                // in progress.
                if self.current_game.is_none() {
                    println!("🤝 {} JOINPRIV", session_code(self));

                    if let Some(game_id) = game_id.decode() {
                        if let Err(e) = self
                            .kafka_commands_in
                            .send(KafkaCommands::JoinPrivateGame(
                                JoinPrivateGameKafkaCommand {
                                    game_id,
                                    client_id: self.client_id,
                                },
                            ))
                            .map_err(|e| ws::Error::from(Box::new(e)))
                        {
                            println!("ERROR on kafka send join private game {:?}", e)
                        }

                        let (events_in, events_out) = client_event_channels();

                        // ..and let the router know we're interested in it,
                        // so that we can receive updates
                        if let Err(e) =
                            self.router_commands_in
                                .send(RouterCommand::JoinPrivateGame {
                                    client_id: self.client_id,
                                    game_id,
                                    events_in,
                                }) {
                            println!(
                                "😠 {} {:<8} sending router command to add client {}",
                                session_code(self),
                                "ERROR",
                                e
                            )
                        }

                        //.. and track the out-channel so we can select! on it
                        self.events_out = Some(events_out);
                    }
                } else {
                    println!("🏴‍☠️ FAILED TO DECODE PRIVATE GAME ID 🏴‍☠️")
                }

                Ok(self.observe())
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
            // EXPIRE timeout has occured, this means that the connection is inactive, let's close
            EXPIRE => {
                if let Some(dur) = self.expire_after.checked_duration_since(Instant::now()) {
                    if dur.as_millis() >= EXPIRE_TIMEOUT_MS.into() {
                        println!("⌛️ {} {:<8} connection", session_code(self), "EXPIRE");
                        self.notify_router_close();
                        return self.ws_out.close(CloseCode::Away);
                    }
                }

                println!(
                    "🤐 {} {:<8} expire timeout",
                    session_code(self),
                    "IGNORED"
                );
                Ok(())
            }
            CHANNEL_RECV => {
                if let Some(eo) = &self.events_out {
                    while let Ok(event) = eo.try_recv() {
                        match event {
                            ClientEvents::OpenGameReply(OpenGameReplyEvent {
                                game_id,
                                reply_to: _,
                                event_id: _,
                            }) => self.current_game = Some(game_id),
                            ClientEvents::GameReady(GameReadyClientEvent {
                                game_id,
                                event_id: _,
                            }) => self.current_game = Some(game_id),
                            _ => (),
                        }

                        self.ws_out.send(serde_json::to_string(&event).unwrap())?;
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
                self.observe();
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
