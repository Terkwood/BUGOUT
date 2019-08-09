import com.fasterxml.jackson.annotation.JsonIgnoreProperties
import serdes.jsonMapper
import java.util.*


typealias GameId = UUID
typealias ReqId = UUID
typealias EventId = UUID

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)
@JsonIgnoreProperties(value = ["gameId", "replyTo", "eventId", "captured"])
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

@JsonIgnoreProperties(value = ["board", "captures", "boardSize", "turn"])
data class GameState(
    val moves: List<MoveEv>, val
    playerUp: Player
) {
    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }

    // TODO
    fun toHistory(gameId: GameId): History =
        History(
            gameId, moves = listOf()
        )

}

data class ProvideHistoryGameState(
    val provideHistory: ProvideHistoryCommand,
    val gameState: GameState
)