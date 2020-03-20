# BUGOUT

🐛 Play Go against your friends,   _over the web!_ 🕸

🚧 Under construction: support for playing against KataGo on a power-efficient dev board 🚧

![BUGOUT Online Go](BUGOUT.jpeg)

## Design

- Powered by [Sabaki](https://sabaki.yichuanshen.de/), [kafka](https://kafka.apache.org/), and [Redis](https://redis.io/) 
- Uses a [fork of Sabaki](https://github.com/Terkwood/Sabaki) for the web UI
- A [public-facing websocket gateway](gateway/README.md) communicates with the browser
- At scale: [Kotlin & Kafka streams adjudicates games](judge/README.md), [announces moves](changelog/README.md), [provides game lobby functionality](game-lobby/README.md), and [implements fair color choice for the players](color-chooser/README.md).
- In low-traffic scenarios: rust and redis streams manages data flow using a tiny memory footprint 

### Kafka streams topologies

This is an example of the data flow used to judge individual moves:

![Kafka streams topo for judge](judge/topology.jpg)

You can view topologies for the various services:

- [color-chooser](color-chooser/topology.jpg)
- [game-lobby](game-lobby/topo.jpg)
- [history-provider](history-provider/topo.jpg)

## Resources

- [Kotlin + Kafka streams](https://blog.ippon.tech/kafka-tutorial-6-kafka-streams-in-kotlin/)
- [Kafka streams quickstart](https://kafka.apache.org/22/documentation/streams/quickstart)
