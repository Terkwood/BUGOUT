import com.fasterxml.jackson.annotation.JsonIgnoreProperties

@JsonIgnoreProperties(ignoreUnknown = true)
data class GameReady(
    val gameId: GameId,
    val sessions: Pair<SessionId, SessionId>
 )