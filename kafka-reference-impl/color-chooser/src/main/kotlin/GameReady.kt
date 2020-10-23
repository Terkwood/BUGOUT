import com.fasterxml.jackson.annotation.JsonIgnoreProperties

/** emitted by game-lobby */
@JsonIgnoreProperties(ignoreUnknown = true)
data class GameReady(val gameId: GameId,
                     val sessions: Pair<SessionId, SessionId>,
                     val eventId: EventId)
