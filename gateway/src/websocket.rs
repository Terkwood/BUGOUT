/// Adapted from https://github.com/housleyjk/ws-rs/blob/master/examples/pong.rs
use std::str::from_utf8;

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
    pub bugout_commands_in: crossbeam_channel::Sender<Commands>,
    pub events_out: Option<crossbeam_channel::Receiver<Events>>,
    pub router_commands_in: crossbeam_channel::Sender<RouterCommand>,
    current_game: Option<GameId>,
}

impl WsSession {
    pub fn new(
        client_id: ClientId,
        ws_out: ws::Sender,
        bugout_commands_in: crossbeam_channel::Sender<Commands>,
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
        }
    }

    fn notify_router_close(&mut self) {
        if let Some(game_id) = self.current_game {
            self.router_commands_in
                .send(RouterCommand::DeleteClient {
                    client_id: self.client_id,
                    game_id,
                })
                .expect("error sending delete client message")
        }
    }
}

impl Handler for WsSession {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        println!("🎫 {} OPEN", short_uuid(self.client_id));

        // schedule a timeout to send a ping every 5 seconds
        self.ws_out.timeout(PING_TIMEOUT_MS, PING)?;
        // schedule a timeout to close the connection if there is no activity for 30 seconds
        self.ws_out.timeout(EXPIRE_TIMEOUT_MS, EXPIRE)?;
        // schedule a timeout to poll for kafka originated
        // events on the crossbeam channel
        self.ws_out.timeout(CHANNEL_RECV_TIMEOUT_MS, CHANNEL_RECV)
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let deserialized: Result<Commands> = serde_json::from_str(&msg.into_text()?)
            .map_err(|_err| ws::Error::new(ws::ErrorKind::Internal, "json"));
        match deserialized {
            Ok(Commands::MakeMove(MakeMoveCommand {
                game_id,
                req_id,
                player,
                coord,
            })) => {
                println!(
                    "{} {} MOVE   {} {:?} {:?}",
                    emoji(&player),
                    short_uuid(self.client_id),
                    short_uuid(game_id),
                    player,
                    coord
                );

                if let Some(c) = self.current_game {
                    if c == game_id {
                        return self
                            .bugout_commands_in
                            .send(Commands::MakeMove(MakeMoveCommand {
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
            Ok(Commands::Beep) => {
                println!("🤖 {} BEEP   ", short_uuid(self.client_id));

                Ok(())
            }
            Ok(Commands::RequestOpenGame(req)) => {
                // For now, set the current game id to
                // the first one that we try to send a
                // command to
                if self.current_game.is_none() {
                    let (events_in, events_out): (
                        crossbeam_channel::Sender<Events>,
                        crossbeam_channel::Receiver<Events>,
                    ) = unbounded();

                    // ..and let the router know we're interested in it,
                    // so that we can receive updates
                    self.router_commands_in
                        .send(RouterCommand::RequestOpenGame {
                            client_id: self.client_id,
                            events_in,
                            req_id: req.req_id,
                        })
                        .expect("error sending router command to add client");

                    //.. and track the out-channel so we can select! on it
                    self.events_out = Some(events_out);
                }
                Ok(())
            }
            Ok(Commands::Reconnect(req)) => unimplemented!(),
            Err(_err) => {
                println!(
                    "💥 {} ERROR  message deserialization failed",
                    short_uuid(self.client_id)
                );
                Ok(())
            }
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!(
            "🚪 {} CLOSE  {} ({:?}) {}",
            short_uuid(self.client_id),
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
        println!("🔥 {} ERROR  {:?}", short_uuid(self.client_id), err,)
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
                self.notify_router_close();
                self.ws_out.close(CloseCode::Away)
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
            self.expire_timeout = Some(timeout)
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
                    short_uuid(self.client_id),
                    (now - pong) as f64 / 1_000_000f64
                );
            } else {
                println!("😐 {} PONG gone wrong", short_uuid(self.client_id));
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
