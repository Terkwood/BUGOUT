import java.util.*

// requests & commands
data class FindPublicGame(val clientId: ClientId, val sessionId: SessionId)

/**
 * Random UUID is used in the case of gateway being lazy
 * and not sending one
 */
data class CreateGame(
    val clientId: ClientId,
    val visibility: Visibility,
    val gameId: GameId = UUID.randomUUID(),
    val sessionId: SessionId,
    val boardSize: Int = FULL_SIZE_BOARD
)

/**
 * This request is issued when someone
 * wants to join a private game
 *
 * @param gameId    the game ID to join
 * @param sessionId the session ID which issued this req
 */
data class JoinPrivateGame(
    val gameId: GameId,
    val clientId: ClientId,
    val sessionId: SessionId
)

// replies & events
/**
 * This event is issued when someone has created
 * a game and is waiting for their opponent to join.
 *
 * @param sessionId The session ID of the individual
 *                  waiting.  This will be used
 *                  downstream to create the GameReady
 *                  event, which requires that both
 *                  players' session IDs are present
 */
data class WaitForOpponent(
    val gameId: GameId,
    val sessionId: SessionId,
    val eventId: EventId = UUID.randomUUID(),
    val visibility: Visibility
)

data class GameReady(
    val gameId: GameId,
    val sessions: Pair<SessionId, SessionId>,
    val eventId: EventId = UUID.randomUUID(),
    val boardSize: Int = FULL_SIZE_BOARD
)

data class PrivateGameRejected(
    val gameId: GameId,
    val clientId: ClientId,
    val eventId: EventId = UUID.randomUUID(),
    val sessionId: SessionId
)

/** This event is emitted from gateway whenever a session disconnects.
 * It drives the cleanup of abandoned games in the lobby */
data class SessionDisconnected(
    val sessionId: SessionId
)
