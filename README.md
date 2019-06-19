# BUGOUT

Multiplayer web implementation of Go/Baduk/Weiqi using Kafka as the backend 🐛

- Use a fork of [Sabaki](https://github.com/SabakiHQ/Sabaki) for the web UI
- [socket.io](https://socket.io/get-started/chat) for the public facing HTTP gateway. There's an [extended example here](https://hackernoon.com/using-kafka-with-nodejs-socketio-and-d3js-to-build-a-real-time-map-b6d3c3eae381)
- [Kotlin](https://blog.ippon.tech/kafka-tutorial-6-kafka-streams-in-kotlin/) + [Kafka streams](https://kafka.apache.org/22/documentation/streams/quickstart) to adjudicate games
