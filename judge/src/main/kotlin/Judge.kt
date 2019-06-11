import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.Consumed
import org.apache.kafka.streams.kstream.KStream
import org.apache.kafka.streams.kstream.Materialized
import org.apache.kafka.streams.kstream.Produced
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

        val moveMadeEventJsonStream: KStream<GameId, String> =
            makeMoveCommandStream.map { _, move ->
                val eventId = UUID.randomUUID()
                KeyValue(
                    move.gameId,
                    jsonMapper.writeValueAsString(
                        MoveMadeEv(
                            gameId = move.gameId,
                            replyTo = move.reqId,
                            eventId = eventId,
                            player = move.player,
                            coord = move.coord
                        )
                    )
                )
            }

        moveMadeEventJsonStream.to(
            MOVE_MODE_EV_TOPIC,
            Produced.with(Serdes.UUID(), Serdes.String())
        )

        // transform moves that are successfully made into a queryable KTable
        val moveMadeEventStream: KStream<GameId, MoveMadeEv> =
            moveMadeEventJsonStream.mapValues { v ->
                jsonMapper.readValue(v, MoveMadeEv::class.java)
            }

        val gameStatesJsonStream =
            moveMadeEventStream.groupByKey().aggregate<GameBoard>(
                { GameBoard() },
                { _, v, board ->
                    board.add(v)
                }, Materialized.with(
                    Serdes.UUID(), Serdes.serdeFrom
                        (GameBoard::class.java)
                )
            ).toStream().map { key, value ->
                KeyValue(
                    key, jsonMapper
                        .writeValueAsString(value)
                )
            }

        gameStatesJsonStream.to(
            GAME_STATES_TOPIC, Produced.with(
                Serdes.UUID(),
                Serdes.String()
            )
        )

        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-judge"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}
