# gateway

WebSocket server which negotiates between BUGOUT frontend and an internal kafka cluster serving the [judge](../judge/README.md).

## Client commands accepted

- Make Move
- Find Public Game
- Create Private Pame
- Join Private Game
- Reconnect
- Provide History
- Beep (client-originated keepalive)

## Overloaded router functionality

Ensures that connected browsers receive updates from kafka, based on a client ID. Maintains crib notes on game states.

## Premade game creation

Sends a number of changelog/gamestate-aggregator empty states on startup, ensuring that there are games available for adjudication.

## Running an example

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
