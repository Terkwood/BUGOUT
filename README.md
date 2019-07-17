# BUGOUT

🐛Play Go against your friends,   _over the web!_ 🕸

![BUGOUT Online Go](BUGOUT.jpeg)

## Design

- Powered by [Sabaki](https://sabaki.yichuanshen.de/) and [kafka](https://kafka.apache.org/)
- Uses a [fork of Sabaki](https://github.com/Terkwood/Sabaki) for the web UI
- A [public-facing websocket gateway](gateway/README.md) communicates with the browser
- [Kotlin & Kafka streams adjudicate games](judge/README.md), announce moves, and [will eventually provide lobby functionality](https://github.com/Terkwood/BUGOUT/issues/42)

## Resources

- [Kotlin + Kafka streams](https://blog.ippon.tech/kafka-tutorial-6-kafka-streams-in-kotlin/)
- [Kafka streams quickstart](https://kafka.apache.org/22/documentation/streams/quickstart)
