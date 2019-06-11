import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.Consumed
import org.apache.kafka.streams.kstream.KStream
import org.apache.kafka.streams.kstream.Produced
import java.util.*

fun main() {
    println("Hello")
    Judge("192.168.65.1:9092").process()
}

class Judge(private val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        val makeMoveCommandJsonStream: KStream<UUID, String> = streamsBuilder
            .stream<UUID, String>(
                MAKE_MOVE_CMD_TOPIC,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )

        val makeMoveCommandStream: KStream<UUID, MakeMoveCmd> =
            makeMoveCommandJsonStream.mapValues { v ->
                jsonMapper.readValue(v, MakeMoveCmd::class.java)
            }

        val moveMadeEventStream: KStream<String, String> =
            makeMoveCommandStream.map { _, move ->
                val eventId = UUID.randomUUID()
                KeyValue(
                    "${move.gameId} $eventId",
                    jsonMapper.writeValueAsString(
                        MoveMadeEv(
                            gameId = move.gameId,
                            reqId = move.reqId,
                            eventId = eventId,
                            player = move.player,
                            coord = move.coord
                        )
                    )
                )
            }

        moveMadeEventStream.to(
            MOVE_MODE_EV_TOPIC,
            Produced.with(Serdes.String(), Serdes.String())
        )

        // TODO link error
        // https://stackoverflow.com/questions/43742423/unsatisfiedlinkerror-on-lib-rocks-db-dll-when-developing-with-kafka-streams
        // TODO you can probably work around this by running in a *nix
        // environment!
        // ... or else, you may need to use an in-memory store to run this on
        // mac, see
        // https://stackoverflow.com/questions/50572237/error-librocksdbjni6770528225908825804-dll-whil-joining-2-streams-or-while-crea
        /*val tinkerBoardStream =
            makeMoveCommandStream.groupByKey().aggregate<GameBoard>(
                { GameBoard() },
                { _, v, board ->
                    board.heedlessAdd(v)
                }
            ).toStream().map { key, value -> KeyValue(key, value) }
        */
        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-judge"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}
