# BUGOUT

Multiplayer web implementation of Go/Baduk/Weiqi using Kafka as the backend 🐛

- Use a [fork of Sabaki](https://github.com/Terkwood/Sabaki) for the web UI
- A [public-facing websocket gateway](gateway/README.md).
- [Kotlin](https://blog.ippon.tech/kafka-tutorial-6-kafka-streams-in-kotlin/) + [Kafka streams](https://kafka.apache.org/22/documentation/streams/quickstart) to adjudicate games
