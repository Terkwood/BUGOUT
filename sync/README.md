# sync service

This service accepts requests for synchronization which
originate from the browser.  It replies with the canonical
history of moves.

In the event that the service layer has missed a move previously
made by the client, the service initiates a MakeMove command
which catches up the service data layer.  In this case, we do not
emit the sync reply event until we hear the move correctly
announced on `bugout-move-made-ev`.

See [the original ticket](https://github.com/Terkwood/BUGOUT/issues/136).

## API

Gateway requests a sync on behalf of the client:

```kt
ReqSyncCmd(
    sessionId, 
    reqId, 
    turn = 2, 
    playerUp = Player.WHITE,
    lastMove = Move(Player.BLACK, turn = 1, coord = Coord(4,4)),    
    gameId = gameId)
```

Sync service responds in one of two ways:

```kt
// No-op: client is already caught up
SyncReplyEv(
    sessionId,
    replyTo = reqId,
    turn = 2,
    playerUp = Player.WHITE,
    moves = listOf(Move(Player.BLACK, turn = 1, coord = Coord(4,4))),
    gameId = gameId
)

// Client is behind, server is ahead
SyncReplyEv(
    sessionId,
    replyTo = reqId,
    turn = 3,
    playerUp = Player.BLACK,
    moves = listOf(
        Move(Player.BLACK, turn = 1, coord = Coord(4,4)),
        Move(Player.WHITE, turn = 2, coord = Coord(10,10)),
    ),
    gameId = gameId
)

// Server is behind, client is ahead.
// This looks the same as a no-op, but in this
// case, sync service will emit a MakeMoveCmd
// to be processed by judge, changelog.
SyncReplyEv(
    sessionId,
    replyTo = reqId,
    turn = 2,
    playerUp = Player.WHITE,
    moves = listOf(Move(Player.BLACK, turn = 1, coord = Coord(4,4))),
    gameId = gameId
)
```
