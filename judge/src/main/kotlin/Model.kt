import java.util.*

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int) {
    init { require (x in 0 .. BOARD_SIZE) }
}

data class MakeMoveCmd(val gameId: UUID,
                       val reqId: UUID,
                       val player: Player,
                       val coord: Coord)

data class MoveMadeEv(val gameId: UUID,
                      val reqId: UUID,
                      val eventId: UUID = UUID.randomUUID(),
                      val player: Player,
                      val coord: Coord)

data class MoveRejectedEv(val gameId: UUID,
                          val reqId: UUID,
                          val player: Player,
                          val coord: Coord)
