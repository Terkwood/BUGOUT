import Topics.GAME_READY
import Topics.GAME_STATES_CHANGELOG
import Topics.GAME_STATES_STORE_NAME
import Topics.MOVE_ACCEPTED_EV
import Topics.MOVE_MADE_EV
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
import java.time.temporal.ChronoUnit
import java.util.*

fun main() {
    Aggregator("kafka:9092").process()
}

class Aggregator(private val brokers: String) {
    fun process() {
        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-gamestates-aggregator"
        props["processing.guarantee"] = "exactly_once"

        waitForTopics(Topics.all, props)

        val streams = KafkaStreams(build(), props)
        streams.start()
    }

    fun build(): Topology {

        val streamsBuilder = StreamsBuilder()
        val moveAccepted: KStream<UUID, MoveMade> =
            streamsBuilder.stream<UUID, String>(
                MOVE_ACCEPTED_EV,
                Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues { v ->
                jsonMapper.readValue(v, MoveMade::class.java)
            }

        val boardSize: KStream<UUID, Int> =
            streamsBuilder.stream<UUID, String>(
                GAME_READY,
                Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues { v ->
                jsonMapper.readValue(v, GameReady::class.java).boardSize
            }


        val pair: KStream<UUID, MoveMadeBoardSize> = moveAccepted
            .join(boardSize,
                { left: MoveMade, right: Int -> MoveMadeBoardSize(left,right)},
                JoinWindows.of(ChronoUnit.DAYS.duration),
                Joined.with(Serdes.UUID(),
                    Serdes.serdeFrom(MoveMadeSer(), MoveMadeDes()),
                    Serdes.Integer()))


        val gameStatesOut: KTable<UUID, GameState> =
            // insight: // https://stackoverflow.com/questions/51966396/wrong-serializers-used-on-aggregate
            pair
                .groupByKey()
                .aggregate(
                    { GameState() },
                    { _, v, gameState ->
                        gameState.add(
                            v.moveMade
                        )
                        // Make sure board size isn't lost from
                        // turn to turn
                        gameState.board.size = v.boardSize
                        gameState
                    },
                    Materialized.`as`<GameId, GameState, KeyValueStore<Bytes,
                            ByteArray>>(
                        GAME_STATES_STORE_NAME
                    )
                        .withKeySerde(Serdes.UUID())
                        .withValueSerde(
                            Serdes.serdeFrom(
                                GameStateSer(),
                                GameStateDes()
                            )
                        )
                )

        gameStatesOut
            .toStream()
            .map { k, v ->
                println("\uD83D\uDCBE          ${k?.toString()?.take(8)} AGGRGATE Turn ${v.turn} PlayerUp ${v.playerUp}")
                KeyValue(k,jsonMapper.writeValueAsString(v))
            }.to(
                GAME_STATES_CHANGELOG,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        val gameStatesIn: KStream<UUID, GameState> =
            streamsBuilder.stream<UUID, String>(
                GAME_STATES_CHANGELOG,
                Consumed.with(Serdes.UUID(), Serdes.String())
                    ).mapValues { v ->
                jsonMapper.readValue(v, GameState::class.java)
            }

        gameStatesIn.filter { _, v ->
                v.moves.isNotEmpty()
            }
            .mapValues { v -> v.moves.last() }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(MOVE_MADE_EV,
                Produced.with(Serdes.UUID(), Serdes.String()))

        return streamsBuilder.build()
    }
    
    private fun waitForTopics(topics: Array<String>, props:
    Properties) {
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
