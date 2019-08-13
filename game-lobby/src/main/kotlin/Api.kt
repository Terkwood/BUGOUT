// requests & commands
data class FindPublicGame(val reqId: ReqId)
data class CreateGame(val reqId: ReqId, val visibility: Visibility)
data class JoinPrivateGame(val gameId: GameId, val reqId: ReqId)

// replies & events
data class WaitForOpponent(val gameId: GameId, val replyTo: ReqId, val eventId: EventId)
data class GameReady(val gameId: GameId, val eventId: EventId)
data class PrivateGameRejected(val gameId: GameId, val replyTo: ReqId)
