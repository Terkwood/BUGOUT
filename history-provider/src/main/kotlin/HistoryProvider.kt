import org.apache.kafka.streams.StreamsBuilder

fun main() {
    HistoryProvider("kafka:9092").process()
}

class HistoryProvider (private val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        // TODO
    }
}
