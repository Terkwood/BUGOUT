import java.time.Instant

data class ProvideHistoryCommand(val gameId: GameId, val reqId: ReqId)

/**
 * An ordered list of moves (FIFO).
 *
 * @property gameId the game associated with this history
 * @property moves an ordered list of moves which have occurred;
 *                 first move is head of the list
 */
data class HistoryProvidedEvent(
    val gameId: GameId,
    val replyTo: ReqId,
    val eventId: EventId,
    val moves: List<Move>,
    val epochMillis: Long = Instant.now().toEpochMilli()
)
