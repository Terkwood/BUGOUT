# BUGOUT gateway

websocket server which negotiates between BUGOUT frontend and an internal kafka cluster serving the [judge](../judge/README.md).

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
