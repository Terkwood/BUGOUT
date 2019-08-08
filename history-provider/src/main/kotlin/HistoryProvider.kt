import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.StreamsBuilder
import java.util.*




fun main() {
    HistoryProvider("kafka:9092").process()
}

class HistoryProvider(private val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        throw NotImplementedError()

        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-history-provider"
        props["processing.guarantee"] = "exactly_once"
        
        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}
