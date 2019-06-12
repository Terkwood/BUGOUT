import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import org.apache.kafka.streams.state.QueryableStoreTypes
import serdes.gameBoardSerde
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
            MOVE_MODE_EV_TOPIC,
            Produced.with(Serdes.UUID(), Serdes.String())
        )

        val gameStatesTable: KTable<GameId, GameBoard> =
            moveMadeEventJsonStream.groupByKey(
                // insight: // https://stackoverflow.com/questions/51966396/wrong-serializers-used-on-aggregate
                Serialized.with(
                    Serdes.UUID(),
                    Serdes.String()
                )
            )
                .aggregate(
                    { GameBoard() },
                    { _, v, list ->
                        list.add(
                            jsonMapper.readValue(
                                v,
                                MoveMadeEv::class.java
                            )
                        )
                        list
                    },
                    Materialized.`as`<GameId, GameBoard, KeyValueStore<Bytes,
                            ByteArray>>(
                        GAME_STATES_STORE_NAME
                    )
                        .withKeySerde(Serdes.UUID())
                        .withValueSerde(gameBoardSerde)
                )

        // This data will be committed to the stream
        // based on `commit.interval.ms`: 30000 ms by default
        // https://docs.confluent.io/current/streams/developer-guide/config-streams.html
        gameStatesTable
            .toStream()
            .mapValues { gameBoard ->
                jsonMapper.writeValueAsString(
                    gameBoard.pieces
                )
            }
            .to(
                GAME_STATES_TOPIC,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-judge"

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
                            GameBoard>()
                )
            store.all().forEach {
                println("${it.key}: ${jsonMapper.writeValueAsString(it.value)}")
            }
        }
    }
}
