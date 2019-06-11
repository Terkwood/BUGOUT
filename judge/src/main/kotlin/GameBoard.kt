import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.Serializer
import java.util.*


class GameBoard {
    private val _board: MutableList<Move> = ArrayList()
    fun add(ev: MoveMadeEv): GameBoard {
        val move = Move(ev.player, ev.coord)
        if (!_board.contains(move))
            _board.add(move)

        return this
    }
}

private val gameBoardSerializer: Serializer<GameBoard> =
    JsonPOJOSerializer<GameBoard>()

private val gameBoardDeserializer: Deserializer<GameBoard> =
    JsonPOJODeserializer()

val gameBoardSerde =
    Serdes.serdeFrom(gameBoardSerializer, gameBoardDeserializer)
//object GameBoardSerde {
// see https://kafka.apache.org/10/documentation/streams/developer-guide/datatypes.html
// see https://github.com/apache/kafka/blob/1.0/streams/examples/src/main/java/org/apache/kafka/streams/examples/pageview/PageViewTypedDemo.java


/*  fun setup() {
      val serdeProps: MutableMap<String, Any> = HashMap()

      serdeProps["JsonPOJOClass"] =
          GameBoard::class.java

      gameBoardSerializer.configure(
          serdeProps, false
      )

      serdeProps["JsonPOJOClass"] = GameBoard::class.java
      gameBoardDeserializer.configure(serdeProps, false)

  }*/
//}
