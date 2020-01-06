import java.util.UUID

data class GameReady(
    val gameId: GameId,
    val sessions: Pair<SessionId, SessionId>
 )