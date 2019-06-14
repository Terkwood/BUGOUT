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
    Thread.sleep(33000)
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
            }.mapValues { v ->
                println(
                    "MAKE MOVE CMD ${v.gameId.short()} ${v
                        .player} ${v
                        .coord}"
                )
                v
            }

        val gameStates: GlobalKTable<GameId, GameState> =
            streamsBuilder
                .globalTable(
                    GAME_STATES_CHANGELOG_TOPIC,
                    Materialized
                        .`as`<GameId, GameState, KeyValueStore<Bytes,
                                ByteArray>>(GAME_STATES_STORE)
                        .withKeySerde(Serdes.UUID())
                        .withValueSerde(
                            Serdes.serdeFrom(
                                GameStateSerializer(),
                                GameStateDeserializer()
                            )
                        )
                )


        val keyJoiner: KeyValueMapper<GameId, MakeMoveCmd, GameId> =
            KeyValueMapper { _: GameId, // left key
                             leftValue: MakeMoveCmd ->
                leftValue.gameId
            }

        val valueJoiner: ValueJoiner<MakeMoveCmd, GameState, MoveCommandGameState> =
            ValueJoiner { leftValue:
                          MakeMoveCmd,
                          rightValue:
                          GameState ->
                MoveCommandGameState(leftValue, rightValue)
            }

        // see https://kafka.apache.org/20/documentation/streams/developer-guide/dsl-api.html#kstream-globalktable-join
        val makeMoveCommandGameStates: KStream<GameId, MoveCommandGameState> =
            makeMoveCommandStream.leftJoin(
                gameStates, keyJoiner,
                valueJoiner
            )

        makeMoveCommandGameStates.mapValues { v ->
            println("oh hey ${v.moveCmd.gameId} turn ${v.gameState.turn}")
        }

        val branches = makeMoveCommandGameStates
            .kbranch({ _, _ -> true })

        val validMakeMoveCommandStream = branches[0]

        // TODO: do some judging

        val relaxedJudgement: KStream<GameId, MoveMadeEv> =
            validMakeMoveCommandStream.map { _, moveCmdGameState ->
                val eventId = UUID.randomUUID()
                val move = moveCmdGameState.moveCmd
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


        relaxedJudgement.mapValues { v ->
            println(
                "relaxed judgement ${v.gameId.short()}: ${v.player} @ ${v
                    .coord} capturing ${v.captured.joinToString { "," }}"
            )
            jsonMapper.writeValueAsString(v)
        }.to(
            MOVE_MADE_EV_TOPIC,
            Produced.with(Serdes.UUID(), Serdes.String())
        )

        val topology = streamsBuilder.build()
        
        println(topology.describe())

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-judge"
        props["processing.guarantee"] = "exactly_once"

        val streams = KafkaStreams(topology, props)
        streams.start()

        println("Judge started")
    }
}
