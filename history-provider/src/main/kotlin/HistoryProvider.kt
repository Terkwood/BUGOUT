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
    HistoryProvider("kafka:9092").process()
}

class HistoryProvider(private val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        // Need global table for joins
        val gameStates: GlobalKTable<GameId, GameState> =
            streamsBuilder
                .globalTable(
                    GAME_STATES_CHANGELOG_TOPIC,
                    Materialized
                        .`as`<GameId, GameState, KeyValueStore<Bytes,
                                ByteArray>>(GAME_STATES_STORE_NAME)
                        .withKeySerde(Serdes.UUID())
                        .withValueSerde(
                            Serdes.serdeFrom(
                                GameStateSerializer(),
                                GameStateDeserializer()
                            )
                        )
                )

        val provideHistoryCommands: KStream<GameId, ProvideHistoryCommand> =
            streamsBuilder.stream<UUID, String>(
                PROVIDE_HISTORY_TOPIC,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    val w = jsonMapper.readValue(
                        v,
                        ProvideHistoryCommand::class.java
                    )

                    println("command $v")

                    w
                }

        val keyJoiner: KeyValueMapper<GameId, ProvideHistoryCommand, GameId> =
            KeyValueMapper { _: GameId, // left key
                             leftValue: ProvideHistoryCommand ->

                println("key joiner")

                leftValue.gameId
            }

        val valueJoiner: ValueJoiner<ProvideHistoryCommand, GameState, ProvideHistoryGameState> =
            ValueJoiner { leftValue:
                          ProvideHistoryCommand,
                          rightValue:
                          GameState ->
                println("value joiner")

                ProvideHistoryGameState(leftValue, rightValue)
            }

        val provideHistoryGameStates: KStream<GameId,
                ProvideHistoryGameState> =
            provideHistoryCommands.leftJoin(gameStates, keyJoiner, valueJoiner)

        val historyProvidedEvent: KStream<GameId, HistoryProvidedEvent> =
            provideHistoryGameStates.map { _, provideHistoryGameState ->
                println("wakey")

                val eventId = UUID.randomUUID()
                val command = provideHistoryGameState.provideHistory
                val gameState = provideHistoryGameState.gameState

                println("... General greetings ...")

                val kv = KeyValue(
                    command.gameId, HistoryProvidedEvent(
                        gameId =
                        command.gameId,
                        replyTo = command.reqId,
                        eventId = eventId,
                        history = gameState.toHistory(command.gameId)
                    )
                )

                println("👻 $kv")

                kv
            }

        historyProvidedEvent.mapValues { v ->
            println("📚          ️${v.gameId.short()} HISTPROV ")
            jsonMapper.writeValueAsString(v)
        }
            .to(
                HISTORY_PROVIDED_TOPIC,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-history-provider"
        props["processing.guarantee"] = "exactly_once"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}
