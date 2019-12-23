/** emitted by game-lobby */
data class GameReady(val gameId: GameId,
                     val sessions: Pair<SessionId, SessionId>,
                     val eventId: EventId)
