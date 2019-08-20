/** emitted by game-lobby */
data class GameReady(val gameId: GameId,
                     val clients: Pair<ClientId, ClientId>,
                     val eventId: EventId)
