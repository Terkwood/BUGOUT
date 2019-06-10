package model

import BOARD_SIZE
import java.util.*

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int) {
    init { require (x in 0 .. BOARD_SIZE) }
}
data class MakeMoveCmd(val gameId: UUID) //, val player: Player, val coord: Coord)
data class MoveMadeEv(val gameId: UUID)//, val player: Player, val coord: Coord)
