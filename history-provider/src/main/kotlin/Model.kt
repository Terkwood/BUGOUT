import serdes.jsonMapper
import java.util.*

typealias GameId = UUID
typealias ReqId = UUID
typealias EventId = UUID

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)
data class MoveEv(val player: Player, val coord: Coord?)
data class Move(val player: Player, val coord: Coord?, val turn: Int)

/**
 * An ordered list of moves (FIFO).
 *
 * @property gameId the game associated with this history
 * @property moves an ordered list of moves which have occurred;
 *                 first move is head of the list
 */
data class History(
    val gameId: GameId,
    val moves: List<Move>
)

// TODO
data class GameState(val gameId: GameId, val moves: List<MoveEv>) {
    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }

    fun toHistory(): History {
        throw NotImplementedError() // TODO
    }
}

data class ProvideHistoryGameState(
    val provideHistory: ProvideHistoryCommand,
    val gameState: GameState
)