import com.fasterxml.jackson.module.kotlin.jacksonTypeRef
import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.Topology
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import serdes.*
import java.util.*

fun main() {
    Judge("kafka:9092").process()
}

class Judge(private val brokers: String) {
    fun process() {

        val topology = build()

        println(topology.describe())

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-judge"
        props["processing.guarantee"] = "exactly_once"

        waitForTopics(ALL_TOPICS, props)

        val streams = KafkaStreams(topology, props)
        streams.start()

        println("Judge started")
    }

    private fun build(): Topology {
        val streamsBuilder = StreamsBuilder()

        val makeMoveCommandStream: KStream<GameId, MakeMoveCmd> =
            streamsBuilder
                .stream<UUID, String>(
                    MAKE_MOVE_CMD_TOPIC,
                    Consumed.with(Serdes.UUID(), Serdes.String())
                ).mapValues { v ->
                    jsonMapper.readValue(v, MakeMoveCmd::class.java)
                }

        makeMoveCommandStream.foreach { _, v ->
            println(
                "\uD83D\uDCE2 game ${v.gameId.short()} MOVE     ${v
                    .player} ${v
                    .coord} (API)"
            )
        }

        val dedupMakeMoveCommandStream: KStream<GameId, MakeMoveCmd> =
            makeMoveCommandStream
                .mapValues { v -> DedupMakeMoveCmd(DoublePlay.No, v) }
                .groupByKey()
                .reduce( { last: DedupMakeMoveCmd, current: DedupMakeMoveCmd ->
                    val isDoublePlay = if (current.makeMoveCmd.player == last.makeMoveCmd.player ) DoublePlay.Yes else DoublePlay.No

                    DedupMakeMoveCmd(isDoublePlay, current.makeMoveCmd)
                } , Materialized.`as`<GameId, DedupMakeMoveCmd, KeyValueStore<Bytes,ByteArray>>(MAKE_MOVE_DEDUP_STORE).withKeySerde(Serdes.UUID()).withValueSerde(Serdes.serdeFrom(KafkaSerializer(),KafkaDeserializer(
                    jacksonTypeRef())))
        ).toStream().filter { _ , v -> v.doublePlay == DoublePlay.No }
                .mapValues { v -> v.makeMoveCmd}

        dedupMakeMoveCommandStream.foreach { _, v ->
            println(
                "\uD83D\uDCE2 game ${v.gameId.short()} MOVE     ${v
                    .player} ${v
                    .coord} (deduped)"
            )
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

        // Distasteful workaround for https://github.com/Terkwood/BUGOUT/issues/228
        val guardNoNullGameState: KStream<GameId, MakeMoveCmd> =
            dedupMakeMoveCommandStream.join(gameStates,
                KeyValueMapper { _: GameId, leftValue: MakeMoveCmd ->
                    leftValue.gameId
                },ValueJoiner { leftValue: MakeMoveCmd,
                                _: GameState ->  leftValue
                })

        val makeMoveCommandGameStates: KStream<GameId, MoveCommandGameState> =
            guardNoNullGameState.join(
                gameStates, keyJoiner,
                valueJoiner
            )

        val branches = makeMoveCommandGameStates
            .kbranch({ _, moveGameState -> moveGameState.isValid() })

        val validMoveGameState = branches[0]

        val moveAcceptedStream: KStream<GameId, MoveMade> =
            validMoveGameState.map { _, moveCmdGameState ->
                val eventId = UUID.randomUUID()
                val move = moveCmdGameState.moveCmd
                val game = moveCmdGameState.gameState
                val captured: List<Coord> = if (move.coord != null) {
                    capturesFor(move.player, move.coord, game.board).toList()
                } else listOf()
                KeyValue(
                    move.gameId,
                    MoveMade(
                        gameId = move.gameId,
                        replyTo = move.reqId,
                        eventId = eventId,
                        player = move.player,
                        coord = move.coord,
                        captured = captured
                    )
                )
            }

        moveAcceptedStream.foreach {
            _, v -> println(
            "⚖️ game ️${v.gameId.short()} ACCEPT   ${v
                .player} @ ${v
                .coord} capturing ${v.captured.joinToString(",")}"
        )
        }

        moveAcceptedStream.mapValues { v ->
            jsonMapper.writeValueAsString(v)
        }.to(
            MOVE_ACCEPTED_EV_TOPIC,
            Produced.with(Serdes.UUID(), Serdes.String())
        )

        return streamsBuilder.build()
    }

    private fun waitForTopics(topics: Array<String>, props: java.util
    .Properties) {
        print("Waiting for topics ")
        val client = AdminClient.create(props)

        var topicsReady = false
        while(!topicsReady) {
            val found = client.listTopics().names().get()

            val diff = topics.subtract(found.filterNotNull())

            topicsReady = diff.isEmpty()

            if (!topicsReady) Thread.sleep(333)
            print(".")
        }

        println(" done!")
    }
}

enum class DoublePlay { No, Yes }
data class DedupMakeMoveCmd(val doublePlay: DoublePlay, val makeMoveCmd: MakeMoveCmd)