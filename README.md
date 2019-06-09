# BUGOUT
Multiplayer web implementation of Go/Baduk/Weiqi using Kafka as the backend 🐛 

- simple lobby... Join game and put you with the first player who is waiting
- react JS for UI. KISS
- Kafka is an audit log of all moves.
- probably node.js and [socket.io](https://socket.io/get-started/chat) for the public facing HTTP gateway. There's an [extended example here](https://hackernoon.com/using-kafka-with-nodejs-socketio-and-d3js-to-build-a-real-time-map-b6d3c3eae381) 
- probably not [pushpin to enable SSE /push](https://hackernoon.com/supercharging-kafka-enable-realtime-web-streaming-by-adding-pushpin-fd62a9809d94) to web client and hide Kafka layer from internet
- write rust Middleware to adjudicate... Command (move black 5 16) vs event
- on game over, replay game moves quickly
- on join game, race to pick a color
- allow handicap

## domain model

Topic : MoveCommands 

```
MakeMove(game_id=0XDEADBEEF, player=Black, x=5, y=12)
```

Topic : MoveEvents

```
MoveMade
MoveRejected
```

## frontend

Use Google cloud auth for simplicity. GCP userid can be tied to games
