import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.Topology
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.processor.*
import org.apache.kafka.streams.state.KeyValueStore
import org.apache.kafka.streams.state.StoreBuilder
import org.apache.kafka.streams.state.Stores
import serdes.*
import java.time.Duration
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

        val makeMoveDeduped: KStream<GameId, MakeMoveCmd> =
            streamsBuilder
                .stream<UUID, String>(
                    MAKE_MOVE_CMD_DEDUP_TOPIC,
                    Consumed.with(Serdes.UUID(), Serdes.String())
                ).mapValues { v ->
                    jsonMapper.readValue(v, MakeMoveCmd::class.java)
                }

        makeMoveDeduped.foreach { _, v ->
            println(
                "\uD83D\uDCE2 game ${v.gameId.short()} MOVE     ${v
                    .player} ${v
                    .coord} (API)"
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
            makeMoveDeduped.join(gameStates,
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

        val topology = streamsBuilder.build()

        // Set up deduplication of Make Move Commands
        val commandSourceName ="Make Move Command API"
        val processorName = "Process: Deduplicate Make Move Commands"
        val sinkName = "Make Move Commands Deduplicated"

        val lastPlayerStoreSupplier: StoreBuilder<KeyValueStore<ByteArray, ByteArray>> = Stores.keyValueStoreBuilder(
            Stores.inMemoryKeyValueStore(MAKE_MOVE_DEDUP_STORE),
                Serdes.ByteArray(),
                Serdes.ByteArray())

        topology.addSource(commandSourceName, MAKE_MOVE_CMD_TOPIC)
            .addProcessor(processorName, ProcessorSupplier {  DedupMakeMoveAPI() } , commandSourceName)
            .addStateStore(lastPlayerStoreSupplier, processorName)
            .addSink(sinkName, MAKE_MOVE_CMD_DEDUP_TOPIC, processorName)

        return topology
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

class DedupMakeMoveAPI : Processor<ByteArray, ByteArray> {
    private var context: ProcessorContext? = null
    private var kvLastPlayer: KeyValueStore<ByteArray,ByteArray?>? = null

    @Suppress("UNCHECKED_CAST")
    override fun init(context: ProcessorContext?) {
        this.context = context
        kvLastPlayer = context?.getStateStore(MAKE_MOVE_DEDUP_STORE) as KeyValueStore<ByteArray,ByteArray?>
        this.context?.schedule(Duration.ofMillis(100), PunctuationType.STREAM_TIME) {
                _ ->
            context.commit()
        }
    }

    override fun process(key: ByteArray?, value: ByteArray?) {
        if (key != null) {
            val makeMoveCmd = jsonMapper.readValue(value, MakeMoveCmd::class.java)

            val lastPlayerBytes = this.kvLastPlayer?.get(key)

            this.kvLastPlayer?.put(key, makeMoveCmd.player.toBytes())

            if (lastPlayerBytes == null || String(lastPlayerBytes).toPlayer() != makeMoveCmd.player){
                context?.forward(key, value)
            }
        }
    }

    override fun close() {}
}

fun String.toPlayer() : Player {
    if (this.isBlank()) return Player.BLACK
    val norm = this.trim().toUpperCase()
    if (norm[0] == 'W') return Player.WHITE
    return Player.BLACK
}

fun Player.toBytes(): ByteArray {
    return when (this) {
        Player.BLACK -> "BLACK".toByteArray()
        Player.WHITE -> "WHITE".toByteArray()
    }
}