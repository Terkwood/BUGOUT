import java.util.*

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int) {
    init {
        require(x in 0..BOARD_SIZE)
    }
}

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

data class Placement(
    val player: Player,
    val turn: Int
)

data class Move(
    val player: Player,
    val coord: Coord?,
    val captures: List<Coord> = ArrayList()
)

data class Capture(
    val turn: Int,
    val pieces: List<Coord>
)

data class Captures(
    val black: List<Capture> = ArrayList(),
    val white: List<Capture> = ArrayList()
)

