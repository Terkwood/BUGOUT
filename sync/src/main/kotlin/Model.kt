import java.util.UUID

typealias GameId = UUID
typealias RequestId = UUID

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)

/**
 * This is Sync service's API-level representation of a move.
 * This matches the shape used by gateway service and the browser client.
 */
data class Move(val player: Player, val coord: Coord?, val turn: Int)

/**
 * This is the global changelog's preferred representation of a move.
 * Note that it omits the turn number.
 */
data class GameStateMove(val player: Player, val coord: Coord?)
