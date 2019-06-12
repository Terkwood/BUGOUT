import java.util.*
import kotlin.collections.HashMap

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)

data class Board(
    val pieces: MutableMap<Coord, Player> = HashMap(),
    val size: Int
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

data class MoveMadeEv(
    val gameId: GameId,
    val replyTo: RequestId,
    val eventId: EventId = UUID.randomUUID(),
    val player: Player,
    val coord: Coord?,
    val captured: List<Coord> = ArrayList()
)

// Signals an invalid move in reply to a client's request
data class MoveRejectedEv(
    val gameId: GameId,
    val replyTo: RequestId,
    val player: Player,
    val coord: Coord
)

data class Move(
    val player: Player,
    val coord: Coord?,
    val captures: List<Coord> = ArrayList()
)

data class Captures(
    var black: Int = 0,
    var white: Int = 0
)

