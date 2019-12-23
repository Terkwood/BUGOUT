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
    val sessionId: SessionId
)

/**
 * This request is issued when someone
 * wants to join a private game
 *
 * @param gameId    the game ID to join
 * @param clientId  the client ID of individual
 *                  who issued this request
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
 * @param clientId The client ID of the individual
 *                  waiting.  This will be used
 *                  downstream to create the GameReady
 *                  event, which requires that both
 *                  players' clientIds are present
 */
data class WaitForOpponent(
    val gameId: GameId,
    val clientId: ClientId,
    val eventId: EventId = UUID.randomUUID(),
    val visibility: Visibility,
    val sessionId: SessionId
)

data class GameReady(
    val gameId: GameId,
    val clients: Pair<ClientId, ClientId>,
    val eventId: EventId = UUID.randomUUID(),
    val sessions: Pair<SessionId, SessionId>
)

data class PrivateGameRejected(
    val gameId: GameId,
    val clientId: ClientId,
    val eventId: EventId = UUID.randomUUID(),
    val sessionId: SessionId
)

/** This event is emitted from gateway whenever a client disconnects.
 * It drives the cleanup of abandoned games in the lobby */
data class ClientDisconnected(
    val clientId: ClientId
)
