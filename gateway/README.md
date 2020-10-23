# gateway

WebSocket server which negotiates between BUGOUT frontend and an internal redis streams instance which brokers communication among the various micro-services.

## Client commands accepted

- Make Move
- Find Public Game
- Create Private Pame
- Join Private Game
- Reconnect
- Provide History
- Req Sync
- Beep (client-originated keepalive)

## Overloaded router functionality

Ensures that connected browsers receive updates from redis, based on a client ID. Maintains crib notes on game states.

## Deprecated: Running an example

In one terminal:

```sh
cargo run
```

In another terminal on the same machine:

```sh
cargo run --example client
```

## Attribution

Thank you to the authors of [ws-rs](https://github.com/housleyjk/ws-rs).  We adapted your examples to fit this use case and appreciate the effort in releasing this library.
