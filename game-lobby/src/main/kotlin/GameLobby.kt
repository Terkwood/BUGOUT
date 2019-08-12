import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.StreamsBuilder
import java.util.*

fun main() {
    TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
    GameLobby("kafka:9092").process()
}

class GameLobby(val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        throw NotImplementedError()

        val topology = streamsBuilder.build()

        println(topology.describe())

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-game-lobby"
        props["processing.guarantee"] = "exactly_once"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}