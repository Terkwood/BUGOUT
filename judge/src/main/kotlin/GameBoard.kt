import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serde
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.Serializer

class GameBoard {

    private val _board: MutableList<Move> = ArrayList()

    fun add(ev: MoveMadeEv): GameBoard {
        val move = Move(ev.player, ev.coord)
        if (!_board.contains(move))
            _board.add(move)

        return this
    }

    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}

private val gameBoardSerializer: Serializer<GameBoard> =
    JsonPOJOSerializer<GameBoard>()

private val gameBoardDeserializer: Deserializer<GameBoard> =
    JsonPOJODeserializer<GameBoard>()

val gameBoardSerde: Serde<GameBoard> =
    Serdes.serdeFrom(gameBoardSerializer, gameBoardDeserializer)
