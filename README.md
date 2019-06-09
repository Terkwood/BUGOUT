# BUGOUT
Multiplayer web implementation of Go/Baduk/Weiqi using Kafka as the backend 🐛 

- simple lobby... Join game and put you with the first player who is waiting
- react JS for UI. KISS
- Kafka is an audit log of all moves.
- maybe pushpin to enable SSE /push to web client and hide Kafka layer from internet
-  write rust Middleware to adjudicate... Command (move black 5 16) vs event
- on game over, replay game moves quickly
- on join game, race to pick a color
- allow handicap

# domain model

Topic : MoveCommands 

```
MakeMove(game_id=0XDEADBEEF, player=Black, x=5, y=12)
```

Topic : MoveEvents

```
MoveMade
MoveRejected
```
