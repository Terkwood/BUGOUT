
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.Consumed
import org.apache.kafka.streams.kstream.KStream

val streamsBuilder = StreamsBuilder()
val gamesStream: KStream<String, String> = streamsBuilder
    .stream<String, String>("bugout-games", Consumed.with(Serdes.String(), Serdes.String()))
