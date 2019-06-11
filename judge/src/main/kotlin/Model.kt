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
    val coord: Coord
)

data class MoveMadeEv(
    val gameId: GameId,
    val replyTo: RequestId,
    val eventId: EventId = UUID.randomUUID(),
    val player: Player,
    val coord: Coord
)

data class MoveRejectedEv(
    val gameId: GameId,
    val replyTo: RequestId,
    val player: Player,
    val coord: Coord
)
