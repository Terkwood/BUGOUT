import java.time.LocalDateTime
import java.time.ZoneOffset

data class ProvideHistoryCommand(val gameId: GameId, val reqId: ReqId)

data class HistoryProvidedEvent(
    val gameId: GameId,
    val replyTo: ReqId,
    val eventId: EventId,
    val history: History,
    val epochMillis: Long = LocalDateTime
        .now(ZoneOffset.UTC).atZone(ZoneOffset.UTC)?.toInstant()?.toEpochMilli()
        ?: 0L
)
