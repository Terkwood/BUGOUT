import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.Consumed
import org.apache.kafka.streams.kstream.KStream
import org.apache.kafka.streams.state.QueryableStoreTypes
import serdes.jsonMapper
import java.util.*

fun main() {
    Judge("kafka:9092").process()
}

class Judge(private val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        val makeMoveCommandJsonStream: KStream<GameId, String> =
            streamsBuilder
                .stream<UUID, String>(
                    MAKE_MOVE_CMD_TOPIC,
                    Consumed.with(Serdes.UUID(), Serdes.String())
                )

        val makeMoveCommandStream: KStream<GameId, MakeMoveCmd> =
            makeMoveCommandJsonStream.mapValues { v ->
                jsonMapper.readValue(v, MakeMoveCmd::class.java)
            }


        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-judge"
        props["processing.guarantee"] = "exactly_once"

        val streams = KafkaStreams(topology, props)
        streams.start()

        // Even though the GAME_STATES_TOPIC stream receives
        // commits infrequently, we can see that the state
        // store itself is updated much more quickly.
        kotlin.concurrent.fixedRateTimer(
            "query",
            initialDelay = 45000, // in case kafka stream thread is starting up
            period = 1000
        ) {
            val store = streams
                .store(
                    GAME_STATES_STORE_NAME,
                    QueryableStoreTypes.keyValueStore<UUID,
                            GameState>()
                )
            store.all().forEach {
                println(
                    "${it.key.toString().take(8)}: ${jsonMapper
                        .writeValueAsString(
                            it
                                .value
                        )}"
                )
            }
        }
    }
}
