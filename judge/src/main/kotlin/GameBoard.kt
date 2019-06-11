import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serde
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.Serializer

class GameBoard {

    val board: MutableList<Move> = ArrayList()

    fun add(ev: MoveMadeEv): GameBoard {
        val move = Move(ev.player, ev.coord)
        if (!board.contains(move))
            board.add(move)

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
