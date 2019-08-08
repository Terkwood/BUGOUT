import serdes.jsonMapper
import java.util.*

typealias GameId = UUID

// TODO
data class GameState(val gameId: GameId) {
    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}

data class Coord(val x: Int, val y: Int)
