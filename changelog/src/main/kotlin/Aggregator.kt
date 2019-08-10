import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
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
        val moveAcceptedJson = streamsBuilder.stream<UUID, String>(
            MOVE_ACCEPTED_EV_TOPIC,
            Consumed.with(Serdes.UUID(), Serdes.String())
        )

        val gameStates = moveAcceptedJson.groupByKey(
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
                            MoveMadeEvent::class.java
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
            )

        gameStates
            .toStream()
            .map { k, v ->
                println("\uD83D\uDCBE          ${k?.toString()?.take(8)} AGGRGATE Turn ${v.turn} PlayerUp ${v.playerUp}")
                KeyValue(k,jsonMapper.writeValueAsString(v))
            }.to(
                GAME_STATES_CHANGELOG_TOPIC,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        gameStates
            .toStream()
            .filter { _, v ->
                v.moves.isNotEmpty()
            }
            .mapValues { v -> v.moves.last() }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(MOVE_MADE_EV_TOPIC,
                Produced.with(Serdes.UUID(), Serdes.String()))

        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-gamestates-aggregator"
        props["processing.guarantee"] = "exactly_once"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}
