import com.fasterxml.jackson.annotation.JsonIgnoreProperties

@JsonIgnoreProperties(ignoreUnknown = true)
data class GameReady(
    val gameId: GameId,
    val sessions: Pair<SessionId, SessionId>
 )

@JsonIgnoreProperties(ignoreUnknown = true)
data class FindPublicGame(val clientId: ClientId, val sessionId: SessionId)

@JsonIgnoreProperties(ignoreUnknown = true)
data class CreateGame(
    val clientId: ClientId,
    val sessionId: SessionId,
    val visibility: Visibility
)

@JsonIgnoreProperties(ignoreUnknown = true)
data class JoinPrivateGame(
    val gameId: GameId,
    val clientId: ClientId,
    val sessionId: SessionId
)