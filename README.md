# BUGOUT

🐛 Play Go against your friends, over the web! 🕸

🤖 _Or play against KataGo AI_ on a power-efficient dev board! 🤖

![BUGOUT Online Go](BUGOUT.jpeg)

Read about the evolution of the project [on terkwood.farm](https://terkwood.farm/tech/BUGOUT/index.html).


## Design

- Allows play against AI using [KataGo](https://github.com/lightvector/KataGo), running on an [NVIDIA Jetson Nano](https://developer.nvidia.com/embedded/jetson-nano-developer-kit) and consuming a mere 5W of power.
- Backend powered by [kafka](https://kafka.apache.org/) and [Redis](https://redis.io/).
- Uses a descendant of [Sabaki](https://sabaki.yichuanshen.de/) for the [web UI](browser/).
- A [public-facing websocket gateway](gateway/README.md) communicates with the browser.
