import java.util.UUID

typealias GameId = UUID
typealias ReqId = UUID
typealias SessionId = UUID
typealias EventId = UUID

enum class Player { BLACK, WHITE }
data class Coord(val x: Int, val y: Int)

/**
 * This is Sync service's API-level representation of a move.
 * This matches the shape used by gateway service and the browser client.
 */
data class Move(val player: Player, val coord: Coord?, val turn: Int)

data class HistProvReply(
    val reqSync: ReqSyncCmd,
    val histProv: HistoryProvidedEv,
    val systemTurn: Int,
    val systemPlayerUp: Player
)

data class SystemMoved(val hist: HistProvReply, val moved: MoveMadeEv)
