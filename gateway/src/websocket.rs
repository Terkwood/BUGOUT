/// Adapted from https://github.com/housleyjk/ws-rs/blob/master/examples/pong.rs
use std::str::from_utf8;

use mio_extras::timer::Timeout;

use uuid::Uuid;
use ws::util::Token;
use ws::{CloseCode, Error, ErrorKind, Frame, Handler, Handshake, Message, OpCode, Result, Sender};

use crate::model::{BugoutMessage, Commands, MakeMoveCommand, MoveMadeEvent};

const PING: Token = Token(1);
const EXPIRE: Token = Token(2);

// WebSocket handler
pub struct WsSession {
    pub client_id: Uuid,
    pub out: Sender,
    pub ping_timeout: Option<Timeout>,
    pub expire_timeout: Option<Timeout>,
    pub kafka_in: crossbeam_channel::Sender<BugoutMessage>,
    pub kafka_out: crossbeam_channel::Receiver<BugoutMessage>,
}

impl Handler for WsSession {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        // schedule a timeout to send a ping every 5 seconds
        self.out.timeout(5_000, PING)?;
        // schedule a timeout to close the connection if there is no activity for 30 seconds
        self.out.timeout(30_000, EXPIRE)
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
            })) => self.out.send(
                serde_json::to_string(&MoveMadeEvent {
                    game_id,
                    reply_to: req_id,
                    player,
                    coord,
                    event_id: Uuid::new_v4(),
                    captured: vec![],
                })
                .unwrap(),
            ),
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
            self.out.cancel(t).unwrap();
        }
        if let Some(t) = self.expire_timeout.take() {
            self.out.cancel(t).unwrap();
        }
    }

    fn on_error(&mut self, err: Error) {
        // Shutdown on any error
        println!("Shutting down WsSession for error: {}", err);
        self.out.shutdown().unwrap();
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            // PING timeout has occured, send a ping and reschedule
            PING => {
                self.out.ping(time::precise_time_ns().to_string().into())?;
                self.ping_timeout.take();
                self.out.timeout(5_000, PING)
            }
            // EXPIRE timeout has occured, this means that the connection is inactive, let's close
            EXPIRE => self.out.close(CloseCode::Away),
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
                self.out.cancel(t)?
            }
            self.expire_timeout = Some(timeout)
        } else if event == PING {
            // This ensures there is only one ping timeout at a time
            if let Some(t) = self.ping_timeout.take() {
                self.out.cancel(t)?
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
        self.out.timeout(30_000, EXPIRE)?;

        // Run default frame validation
        DefaultHandler.on_frame(frame)
    }
}

// For accessing the default handler implementation
struct DefaultHandler;

impl Handler for DefaultHandler {}
