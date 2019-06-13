import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
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
            makeMoveCommandStream.map { _, move ->
                val eventId = UUID.randomUUID()
                KeyValue(
                    move.gameId,
                    MoveMadeEv(
                        gameId = move.gameId,
                        replyTo = move.reqId,
                        eventId = eventId,
                        player = move.player,
                        coord = move.coord
                    )
                )

            }

        val moveMadeEventJsonStream: KStream<GameId, String> =
            moveMadeEventStream.mapValues { move ->
                jsonMapper.writeValueAsString(
                    move
                )
            }

        moveMadeEventJsonStream.to(
            MOVE_MADE_EV_TOPIC,
            Produced.with(Serdes.UUID(), Serdes.String())
        )

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
            )

        gameStates
            .toStream()
            .mapValues { v ->
                println("game-states changelog $v")
                jsonMapper.writeValueAsString(v)
            }.to(
                GAME_STATES_CHANGELOG_TOPIC,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

    }
}
