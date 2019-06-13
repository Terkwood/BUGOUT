import java.util.*
import kotlin.collections.HashMap

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)

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


data class MakeMoveCmd(
    val gameId: GameId,
    val reqId: RequestId,
    val player: Player,
    val coord: Coord?
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


