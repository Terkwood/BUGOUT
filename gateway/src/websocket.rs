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
    pub bugout_commands_in: crossbeam_channel::Sender<ClientCommands>,
    pub events_out: Option<crossbeam_channel::Receiver<Events>>,
    pub router_commands_in: crossbeam_channel::Sender<RouterCommand>,
    pub current_game: Option<GameId>,
    pub expire_after: std::time::Instant,
}

impl WsSession {
    pub fn new(
        client_id: ClientId,
        ws_out: ws::Sender,
        bugout_commands_in: crossbeam_channel::Sender<ClientCommands>,
        router_commands_in: crossbeam_channel::Sender<RouterCommand>,
    ) -> WsSession {
        WsSession {
            client_id,
            ws_out,
            ping_timeout: None,
            expire_timeout: None,
            channel_recv_timeout: None,
            bugout_commands_in,
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
        let deserialized: Result<ClientCommands> = serde_json::from_str(&msg.into_text()?)
            .map_err(|_err| ws::Error::new(ws::ErrorKind::Internal, "json"));
        match deserialized {
            Ok(ClientCommands::MakeMove(MakeMoveCommand {
                game_id,
                req_id,
                player,
                coord,
            })) => {
                println!(
                    "{} {} MOVE   {:?} {:?}",
                    emoji(&player),
                    session_code(self),
                    player,
                    coord
                );

                if let Some(c) = self.current_game {
                    if c == game_id {
                        return self
                            .bugout_commands_in
                            .send(ClientCommands::MakeMove(MakeMoveCommand {
                                game_id,
                                req_id,
                                player,
                                coord,
                            }))
                            .map_err(|e| ws::Error::from(Box::new(e)));
                    }
                }

                Ok(())
            }
            Ok(ClientCommands::Beep) => {
                println!("🤖 {} BEEP   ", session_code(self));

                Ok(())
            }
            Ok(ClientCommands::RequestOpenGame(req)) => {
                // Tentatively ignore this request if we already have a game
                // in progress.  Not sure this is perfect, though.
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
                            "😠 {} ERROR  sending router command to add client {}",
                            session_code(self),
                            e
                        )
                    }

                    //.. and track the out-channel so we can select! on it
                    self.events_out = Some(events_out);
                }
                Ok(())
            }
            Ok(ClientCommands::Reconnect(ReconnectCommand { game_id, req_id })) => {
                // accept whatever game_id the client shares with us
                self.current_game = Some(game_id);

                println!("🔌 {} RECONN ", session_code(self));
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
                        "😫 {} ERROR   sending router command to reconnect client {:?}",
                        session_code(self),
                        e
                    )
                }

                //.. and track the out-channel so we can select! on it
                self.events_out = Some(events_out);

                Ok(())
            }
            Err(_err) => {
                println!(
                    "💥 {} ERROR  message deserialization failed",
                    session_code(self)
                );
                Ok(())
            }
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!(
            "🚪 {} CLOSE  {} ({:?}) {}",
            session_code(self),
            short_time(),
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
        println!("🔥 {} ERROR  {:?}", session_code(self), err,)
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
                        println!("⌛️ {} EXPIRE  CONN", session_code(self));
                        self.notify_router_close();
                        return self.ws_out.close(CloseCode::Away);
                    }
                }

                println!("🤐 {} IGNORED  expire timeout", session_code(self));
                Ok(())
            }
            CHANNEL_RECV => {
                if let Some(eo) = &self.events_out {
                    while let Ok(event) = eo.try_recv() {
                        match event {
                            Events::OpenGameReply(OpenGameReplyEvent {
                                game_id,
                                reply_to: _,
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
                let now = time::precise_time_ns();
                println!(
                    "🏓 {} PING   PONG   ({:.3}ms)",
                    session_code(self),
                    (now - pong) as f64 / 1_000_000f64
                );
            } else {
                println!("😐 {} PONG gone wrong", session_code(self));
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
    crossbeam_channel::Sender<Events>,
    crossbeam_channel::Receiver<Events>,
) {
    unbounded()
}
