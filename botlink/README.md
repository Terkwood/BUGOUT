# botlink service

This websocket endpoint brokers communication between an 
NVIDIA Jetson Nano dev board running katago, and the cloud 
deployment for BUGOUT.

## How it works

It listens to `bugout-attach-bot-ev` redis stream for `game_id + player`
combinations and will subsequently respond to game state changelog events
which concern the player.

Each time the game state changelog emits an event such that it is
the given player's turn, a `ComputeMove` request will be sent to the
NVIDIA Jetson Nano board (tinybrain) over websocket.

Eventually the tinybrain will respond with a `MoveComputed`, which is
subsequently written to the `bugout-make-move-command` stream, and
processed as a normal by `micro-judge`.
