# BUGOUT

🐛 Play Go against your friends, over the web! 🕸

🤖 _Or play against KataGo AI_ on a power-efficient dev board! 🤖

![BUGOUT Online Go](BUGOUT.jpeg)


## Design

- Allows play against AI using [KataGo](https://github.com/lightvector/KataGo), running on an [NVIDIA Jetson Nano](https://developer.nvidia.com/embedded/jetson-nano-developer-kit) and consuming a mere 5W of power.
- Backend powered by [Redis](https://redis.io/).
- Uses a descendant of [Sabaki](https://sabaki.yichuanshen.de/) for the [web UI](browser/).
- A [public-facing websocket gateway](gateway/README.md) communicates with the browser.

## Marching to Production 

BUGOUT is nearing its production release, at which point we will publish a playable website address. 

Keep an eye on the [pinned issues](https://github.com/Terkwood/BUGOUT/issues) in this github repository if you're interested in the progress toward the release. Primarily we are finishing optimizations to the Multiplayer capability and adding an "easy mode" for playing against the AI. 

## Getting Started

BUGOUT relies on [docker-compose](https://docs.docker.com/compose/install/) to run _most_ of its services.

```sh
docker-compose up
```

If you're hacking BUGOUT, you will want to host the
web application on your local machine.  You need to [install
npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm).  Then:

```sh
cd browser
npm run watch
```

The tinybrain utility currently does not use docker-compose.  This utility wraps [KataGo](https://github.com/lightvector/KataGo)
AI and allows it to communicate with the rest of the backend services.  We run it using a [systemd script](./tinybrain/tinybrain.service).
