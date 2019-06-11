import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serializer
import java.util.*


class GameBoard {
    private val _board: MutableMap<Coord, Player> = HashMap()
    fun add(move: MoveMadeEv): GameBoard {
        if (!_board.containsKey(move.coord))
            _board[move.coord] = move.player

        return this
    }
}

object GameBoardSerde {
    // see https://kafka.apache.org/10/documentation/streams/developer-guide/datatypes.html
    // see https://github.com/apache/kafka/blob/1.0/streams/examples/src/main/java/org/apache/kafka/streams/examples/pageview/PageViewTypedDemo.java
    val gameBoardSerializer: Serializer<GameBoard> =
        JsonPOJOSerializer<GameBoard>()

    val gameBoardDeserializer: Deserializer<GameBoard> =
        JsonPOJODeserializer()

    fun setup() {
        val serdeProps: MutableMap<String, Any> = HashMap()

        serdeProps["JsonPOJOClass"] =
            GameBoard::class.java

        gameBoardSerializer.configure(
            serdeProps, false
        )

        serdeProps[
                "JsonPOJOClass"] = GameBoard::class.java
        gameBoardDeserializer.configure(serdeProps, false)
    }
}
