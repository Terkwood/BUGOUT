import java.time.Instant

data class ProvideHistoryCommand(val gameId: GameId, val reqId: ReqId)

data class HistoryProvidedEvent(
    val gameId: GameId,
    val replyTo: ReqId,
    val eventId: EventId,
    val history: History,
    val epochMillis: Long = Instant.now().toEpochMilli()
)
