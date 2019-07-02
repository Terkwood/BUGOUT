/// Adapted from https://github.com/housleyjk/ws-rs/blob/master/examples/pong.rs
use std::str::from_utf8;

use mio_extras::timer::Timeout;

use ws::util::Token;
use ws::{CloseCode, Error, ErrorKind, Frame, Handler, Handshake, Message, OpCode, Result, Sender};

use crate::model::*;

const PING: Token = Token(1);
const EXPIRE: Token = Token(2);

// WebSocket handler
pub struct WsSession {
    pub client_id: ClientId,
    pub ws_out: Sender,
    pub ping_timeout: Option<Timeout>,
    pub expire_timeout: Option<Timeout>,
    pub command_in: crossbeam_channel::Sender<Commands>,
    pub events_in: crossbeam_channel::Receiver<Events>,
    current_game: Option<GameId>,
}

impl WsSession {
    pub fn new(
        client_id: ClientId,
        ws_out: ws::Sender,
        command_in: crossbeam_channel::Sender<Commands>,
        events_in: crossbeam_channel::Receiver<Events>,
    ) -> WsSession {
        WsSession {
            client_id,
            ws_out,
            ping_timeout: None,
            expire_timeout: None,
            command_in,
            events_in,
            current_game: None,
        }
    }
}

impl Handler for WsSession {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        // schedule a timeout to send a ping every 5 seconds
        self.ws_out.timeout(5_000, PING)?;
        // schedule a timeout to close the connection if there is no activity for 30 seconds
        self.ws_out.timeout(30_000, EXPIRE)
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("WsSession got message '{}'. ", msg);
        let deserialized: Result<Commands> = serde_json::from_str(&msg.into_text()?)
            .map_err(|_err| ws::Error::new(ws::ErrorKind::Internal, "json"));
        match deserialized {
            Ok(Commands::MakeMove(MakeMoveCommand {
                game_id,
                req_id,
                player,
                coord,
            })) => {
                // For now, set the current game id to
                // the first one that we try to send a
                // command to
                if self.current_game.is_none() {
                    self.current_game = Some(game_id);
                }

                if let Some(c) = self.current_game {
                    if c == game_id {
                        return self
                            .command_in
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
            Err(e) => {
                println!("Error deserializing {:?}", e);
                Ok(())
            }
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("WebSocket closing ({:?}) {}", code, reason);

        // Clean up timeouts when connections close
        if let Some(t) = self.ping_timeout.take() {
            self.ws_out.cancel(t).unwrap();
        }
        if let Some(t) = self.expire_timeout.take() {
            self.ws_out.cancel(t).unwrap();
        }
    }

    fn on_error(&mut self, err: Error) {
        // Shutdown on any error
        println!("Shutting down WsSession for error: {}", err);
        self.ws_out.shutdown().unwrap();
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            // PING timeout has occured, send a ping and reschedule
            PING => {
                self.ws_out
                    .ping(time::precise_time_ns().to_string().into())?;
                self.ping_timeout.take();
                self.ws_out.timeout(5_000, PING)
            }
            // EXPIRE timeout has occured, this means that the connection is inactive, let's close
            EXPIRE => self.ws_out.close(CloseCode::Away),
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
            // This ensures there is only one ping timeout at a time
            if let Some(t) = self.ping_timeout.take() {
                self.ws_out.cancel(t)?
            }
            self.ping_timeout = Some(timeout)
        }

        Ok(())
    }

    fn on_frame(&mut self, frame: Frame) -> Result<Option<Frame>> {
        // If the frame is a pong, print the round-trip time.
        // The pong should contain data from out ping, but it isn't guaranteed to.
        if frame.opcode() == OpCode::Pong {
            if let Ok(pong) = from_utf8(frame.payload())?.parse::<u64>() {
                let now = time::precise_time_ns();
                println!("RTT is {:.3}ms.", (now - pong) as f64 / 1_000_000f64);
            } else {
                println!("Received bad pong.");
            }
        }

        // Some activity has occured, so reset the expiration
        self.ws_out.timeout(30_000, EXPIRE)?;

        // Run default frame validation
        DefaultHandler.on_frame(frame)
    }
}

// For accessing the default handler implementation
struct DefaultHandler;

impl Handler for DefaultHandler {}
