
import model.MakeMoveCmd
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.Consumed
import org.apache.kafka.streams.kstream.KStream
import org.apache.kafka.streams.kstream.Produced
import java.util.*

fun main() {
    Judge("0.0.0.0:9092").process()
}

class Judge(private val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        val makeMoveCommandJsonStream: KStream<String, String> = streamsBuilder
            .stream<String, String>(MAKE_MOVE_CMD_TOPIC, Consumed.with(Serdes.String(), Serdes.String()))

        val makeMoveCommandStream: KStream<String, MakeMoveCmd> = makeMoveCommandJsonStream.mapValues { v ->
            jsonMapper.readValue(v, MakeMoveCmd::class.java)
        }

        val moveMadeEventStream: KStream<String, String> = makeMoveCommandStream.map { _, move ->
            KeyValue("${move.gameId}", "moved")
        }

        moveMadeEventStream.to(MOVE_MODE_EV_TOPIC, Produced.with(Serdes.String(), Serdes.String()))

        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-judge"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}
