import com.fasterxml.jackson.annotation.JsonIgnoreProperties
import com.fasterxml.jackson.annotation.JsonTypeInfo
import java.util.*
import kotlin.collections.HashMap

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)

const val FULL_BOARD_SIZE = 19
data class Board(
    val pieces: MutableMap<Coord, Player> = HashMap(),
    val size: Int = FULL_BOARD_SIZE
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
 * We ignore the type property used by gateway for deserialization.
 * We emit the type property on serialization.
 */
@JsonTypeInfo(
    use = JsonTypeInfo.Id.NAME,
    include = JsonTypeInfo.As.PROPERTY,
    property = "type",
    defaultImpl = MoveMadeEvent::class,
    visible = true
)
@JsonIgnoreProperties(value = ["type"])
data class MoveMadeEvent(
    val gameId: GameId,
    val replyTo: RequestId,
    val eventId: EventId = UUID.randomUUID(),
    val player: Player,
    val coord: Coord?,
    val captured: List<Coord> = ArrayList()
)
