import com.fasterxml.jackson.annotation.JsonIgnoreProperties
import serdes.jsonMapper
import java.util.*


typealias GameId = UUID
typealias ReqId = UUID
typealias EventId = UUID

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)
@JsonIgnoreProperties(value = ["gameId", "replyTo", "eventId", "captured",
    "type"])
data class MoveEv(val player: Player, val coord: Coord?)

data class Move(val player: Player, val coord: Coord?, val turn: Int)

@JsonIgnoreProperties(value = ["board", "captures", "boardSize", "turn"])
data class GameState(
    val moves: List<MoveEv>?, val
    playerUp: Player
) {
    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }

    fun toHistory(): List<Move> = this.moves?.withIndex()
        ?.map { (index, moveEv) -> Pair(index + 1, moveEv) }
        ?.map { (turn, moveEv) ->
            Move(
                player = moveEv.player,
                coord = moveEv.coord,
                turn = turn
            )
        } ?: listOf()

}

data class ProvideHistoryGameState(
    val provideHistory: ProvideHistoryCommand,
    val gameState: GameState
)
