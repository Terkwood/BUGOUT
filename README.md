# BUGOUT

🐛 Play Go against your friends, over the web! 🕸

🤖 _Or play against KataGo AI_ on a power-efficient dev board! 🤖

![BUGOUT Online Go](BUGOUT.jpeg)

Read about the evolution of the project [on terkwood.farm](https://terkwood.farm/tech/BUGOUT/index.html).


## Design

- Allows play against AI using [KataGo](https://github.com/lightvector/KataGo), running on an [NVIDIA Jetson Nano](https://developer.nvidia.com/embedded/jetson-nano-developer-kit) and consuming a mere 5W of power.
- Backend powered by [kafka](https://kafka.apache.org/) and [Redis](https://redis.io/).
- Uses a [descendant of Sabaki](browser/) for the web UI.
- A [public-facing websocket gateway](gateway/README.md) communicates with the browser.
- A reference implementation for the multiplayer system: [Kotlin & Kafka streams adjudicates games](judge/README.md), [announces moves](changelog/README.md), [provides game lobby functionality](game-lobby/README.md), and [implements fair color choice for the players](color-chooser/README.md).
- Real Soon Now: backend completely rewritten with rust, using redis streams to manage data flow.
