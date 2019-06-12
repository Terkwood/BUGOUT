import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serde
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.Serializer

class GameBoard {
    // the pieces are tracked in order
    val pieces: MutableMap<Coord, Player> = HashMap()

    var captures = Captures()

    var turn: Int = 1

    var playerUp: Player = Player.BLACK

    fun add(ev: MoveMadeEv): GameBoard {
        if (ev.coord != null) {
            pieces[ev.coord] = ev.player
            ev.captured.forEach { coord ->
                pieces.remove(coord)
                when (ev.player) {
                    Player.BLACK -> captures.black = captures.black + 1
                    Player.WHITE -> captures.white = captures.white + 1
                }
            }
        }

        turn++

        playerUp = when (playerUp) {
            Player.BLACK -> Player.WHITE
            Player.WHITE -> Player.BLACK
        }

        return this
    }

    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}

private val gameBoardSerializer: Serializer<GameBoard> =
    GameBoardSerializer()

private val gameBoardDeserializer: Deserializer<GameBoard> =
    GameBoardDeserializer()

val gameBoardSerde: Serde<GameBoard> =
    Serdes.serdeFrom(gameBoardSerializer, gameBoardDeserializer)
