import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import org.apache.kafka.streams.state.QueryableStoreTypes
import serdes.GameStateDeserializer
import serdes.GameStateSerializer
import serdes.jsonMapper
import java.util.*

fun main() {
    Aggregator("kafka:9092").process()
}

class Aggregator(private val brokers: String) {
    fun process() {

        val streamsBuilder = StreamsBuilder()
        val moveMadeEventJsonStream = streamsBuilder.stream<UUID, String>(
            MOVE_MADE_EV_TOPIC,
            Consumed.with(Serdes.UUID(), Serdes.String())
        )

        moveMadeEventJsonStream.mapValues { v ->
            println("Hello JSON stream $v")
            v
        }

        // transform pieces that are successfully made into a queryable KTable
        val moveMadeEventStream: KStream<GameId, MoveMadeEv> =
            moveMadeEventJsonStream.mapValues {  move ->
                jsonMapper.readValue(move, MoveMadeEv::class.java)
            }.mapValues { v ->
                println("move made in ${v.gameId}: ${v.player} @ ${v.coord} with captures: ${v.captured.joinToString { "," }}")
                v
            }


        val gameStates = moveMadeEventJsonStream.groupByKey(
            // insight: // https://stackoverflow.com/questions/51966396/wrong-serializers-used-on-aggregate
            Serialized.with(
                Serdes.UUID(),
                Serdes.String()
            )
        )
            .aggregate(
                { GameState() },
                { _, v, list ->
                    list.add(
                        jsonMapper.readValue(
                            v,
                            MoveMadeEv::class.java
                        )
                    )
                    list
                },
                Materialized.`as`<GameId, GameState, KeyValueStore<Bytes,
                        ByteArray>>(
                    GAME_STATES_STORE_NAME
                )
                    .withKeySerde(Serdes.UUID())
                    .withValueSerde(
                        Serdes.serdeFrom(
                            GameStateSerializer(),
                            GameStateDeserializer()
                        )
                    )
            ).mapValues{ v ->
                println("state store: PlayerUp ${v.playerUp}, ${v.board.pieces.size} pieces, turn ${v.turn})")
                v
            }

        gameStates
            .toStream()
            .mapValues { v ->
                println("game-states changelog $v")
                jsonMapper.writeValueAsString(v)
            }.to(
                GAME_STATES_CHANGELOG_TOPIC,
                Produced.with(Serdes.UUID(), Serdes.String())
            )


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
