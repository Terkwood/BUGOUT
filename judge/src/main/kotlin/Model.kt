import com.fasterxml.jackson.annotation.JsonIgnoreProperties
import com.fasterxml.jackson.annotation.JsonTypeInfo
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
typealias EventId = UUID

data class MakeMoveCmd(
    val gameId: GameId,
    val reqId: RequestId,
    val player: Player,
    val coord: Coord?
)

/**
 * An event signalling the acceptance of a move.
 * JSON type field must be populated so that
 * gateway knows how to deserialize this
 */
@JsonTypeInfo(
    use = JsonTypeInfo.Id.NAME,
    include = JsonTypeInfo.As.PROPERTY,
    property = "type",
    defaultImpl = MoveMadeEvent::class,
    visible = true
)
@JsonIgnoreProperties(value = ["type"]) // madness, we have to ignore it on deser
data class MoveMadeEvent(
    val gameId: GameId,
    val replyTo: RequestId,
    val eventId: EventId = UUID.randomUUID(),
    val player: Player,
    val coord: Coord?,
    val captured: List<Coord> = ArrayList()
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

data class MoveCommandGameState(
    val moveCmd: MakeMoveCmd,
    val gameState: GameState
)
