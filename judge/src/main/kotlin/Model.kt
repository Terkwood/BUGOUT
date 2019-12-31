import java.util.*
import kotlin.collections.HashMap

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)

const val FULL_BOARD_SIZE = 19
data class Board(
    val pieces: Map<Coord, Player> = HashMap(),
    val size: Int = FULL_BOARD_SIZE
)

data class Captures(
    var black: Int = 0,
    var white: Int = 0
)

typealias GameId = UUID
typealias RequestId = UUID
typealias EventId = UUID

data class MakeMoveCmd(
    val gameId: GameId,
    val reqId: RequestId,
    val player: Player,
    val coord: Coord?
)

/**
 * An event signalling the acceptance of a move.
 */
data class MoveMade(
    val gameId: GameId,
    val replyTo: RequestId,
    val eventId: EventId = UUID.randomUUID(),
    val player: Player,
    val coord: Coord?,
    val captured: List<Coord> = ArrayList()
)


/** Signals an invalid move in reply to a client's request */
data class MoveRejectedEv(
    val gameId: GameId,
    val replyTo: RequestId,
    val player: Player,
    val coord: Coord
)

data class MoveCommandGameState(
    val moveCmd: MakeMoveCmd,
    val gameState: GameState
)
