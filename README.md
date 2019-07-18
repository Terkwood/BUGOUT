# BUGOUT

🐛Play Go against your friends,   _over the web!_ 🕸

![BUGOUT Online Go](BUGOUT.jpeg)

## Design

- Powered by [Sabaki](https://sabaki.yichuanshen.de/) and [kafka](https://kafka.apache.org/)
- Uses a [fork of Sabaki](https://github.com/Terkwood/Sabaki) for the web UI
- A [public-facing websocket gateway](gateway/README.md) communicates with the browser
- [Kotlin & Kafka streams adjudicates games](judge/README.md), [announces moves](changelog/README.md), and [will eventually provide lobby functionality](https://github.com/Terkwood/BUGOUT/issues/42)

## Data Flow

Judge emits move data to `bugout-move-accepted-ev`, which is an input to changelog.  Changelog emits moves to `bugout-move-made-ev` after recording them.  Gateway listens to `bugout-move-made-ev`.

## Resources

- [Kotlin + Kafka streams](https://blog.ippon.tech/kafka-tutorial-6-kafka-streams-in-kotlin/)
- [Kafka streams quickstart](https://kafka.apache.org/22/documentation/streams/quickstart)
