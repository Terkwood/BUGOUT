# BUGOUT

🐛Play Go against your friends,   _over the web!_ 🕸

![BUGOUT Online Go](BUGOUT.jpeg)

## Design

- Powered by [Sabaki](https://sabaki.yichuanshen.de/) and [kafka](https://kafka.apache.org/)
- Uses a [fork of Sabaki](https://github.com/Terkwood/Sabaki) for the web UI
- A [public-facing websocket gateway](gateway/README.md) communicates with the browser
- [Kotlin & Kafka streams adjudicates games](judge/README.md), [announces moves](changelog/README.md), [provides game lobby functionality](game-lobby/README.md), and [implements fair color choice for the players](color-chooser/README.md).
- [A microservice to provide game history on request](history-provider/README.md)

## Data Flow

Judge emits move data to `bugout-move-accepted-ev`, which is an input to changelog.  Changelog emits moves to `bugout-move-made-ev` after recording them.  Gateway listens to `bugout-move-made-ev`.

## Running in AWS

[This system is slowly being migrated to a more cost-efficient design](https://github.com/Terkwood/BUGOUT/issues/75).  For now, you can run a t3.large box in AWS to host Kafka.  We provide a simple script to help with this:

```sh
sh admin/start-kafka-host.sh
```

## Resources

- [Kotlin + Kafka streams](https://blog.ippon.tech/kafka-tutorial-6-kafka-streams-in-kotlin/)
- [Kafka streams quickstart](https://kafka.apache.org/22/documentation/streams/quickstart)
