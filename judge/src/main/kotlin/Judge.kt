import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import java.util.*

fun main() {
    Judge("kafka:9092").process()
}

class Judge(private val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        val makeMoveCommandJsonStream: KStream<GameId, String> = streamsBuilder
            .stream<UUID, String>(
                MAKE_MOVE_CMD_TOPIC,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )

        val makeMoveCommandStream: KStream<GameId, MakeMoveCmd> =
            makeMoveCommandJsonStream.mapValues { v ->
                jsonMapper.readValue(v, MakeMoveCmd::class.java)
            }


        // transform moves that are successfully made into a queryable KTable
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

        // TODO experimental
        gameStatesTable
            .toStream()
            .mapValues { gameBoard ->
                jsonMapper.writeValueAsString(
                    gameBoard
                        .board
                )
            }
            .to(
                GAME_STATES_TOPIC,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        // see https://cwiki.apache.org/confluence/display/KAFKA/Kafka+Stream+Usage+Patterns
        /* val gameStatesJsonTable: KTable<GameId, String> =
             gameStatesTable.mapValues { list ->
                 jsonMapper
                     .writeValueAsString(list)
             }

         gameStatesJsonTable to GAME_STATES_TOPIC*/


        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-judge"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}
