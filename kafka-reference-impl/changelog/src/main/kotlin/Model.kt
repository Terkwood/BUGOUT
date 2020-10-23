import com.fasterxml.jackson.annotation.JsonIgnoreProperties
import serdes.jsonMapper
import java.util.*
import kotlin.collections.HashMap

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)

const val FULL_BOARD_SIZE = 19
data class Board(
    val pieces: MutableMap<Coord, Player> = HashMap(),
    var size: Int = FULL_BOARD_SIZE
)

data class Captures(
    var black: Int = 0,
    var white: Int = 0
)

typealias GameId = UUID
typealias RequestId = UUID
typealias EventId = UUID

/**
 * An event signaling that a move has been made.
 */
data class MoveMade(
    val gameId: GameId,
    val replyTo: RequestId,
    val eventId: EventId = UUID.randomUUID(),
    val player: Player,
    val coord: Coord?,
    val captured: List<Coord> = ArrayList()
){
    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}

/** Board size and handicaps */
@JsonIgnoreProperties(ignoreUnknown = true)
data class GameReady(
    val boardSize: Int = FULL_BOARD_SIZE
){
    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}

data class MoveMadeBoardSize (
    val moveMade: MoveMade,
    val boardSize: Int
)