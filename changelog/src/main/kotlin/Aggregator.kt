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
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import serdes.GameStateDeserializer
import serdes.GameStateSerializer
import serdes.jsonMapper
import java.time.temporal.ChronoUnit
import java.util.*

fun main() {
    Aggregator("kafka:9092").process()
}

class Aggregator(private val brokers: String) {
    fun process() {

        val streamsBuilder = StreamsBuilder()
        val moveAccepted: KStream<UUID, MoveMade> =
            streamsBuilder.stream<UUID, String>(
                MOVE_ACCEPTED_EV,
                Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues { v ->
                println("MOVE ACCEPTED $v")
                jsonMapper.readValue(v, MoveMade::class.java)}

        val gameReady: KStream<UUID, GameReady> =
            streamsBuilder.stream<UUID, String>(
                    GAME_READY,
                    Consumed.with(Serdes.UUID(), Serdes.String())
            ).map { k,v ->
                println("GAME READY $k -> $v")
                KeyValue(k,jsonMapper.readValue(v, GameReady::class.java))}

        val funTimes = gameReady.mapValues{ v ->
            println("more fun times ")
            v
        }

        /*
        val pair = moveAccepted.join(gameReady,
            { left: MoveMade, right: GameReady -> MoveMadeGameReady(left,right)},JoinWindows.of(
            ChronoUnit.YEARS.duration)
        )
       
         */

        val gameStates: KTable<UUID, GameState> =
            // insight: // https://stackoverflow.com/questions/51966396/wrong-serializers-used-on-aggregate
            moveAccepted
                .groupByKey()
                .aggregate(
                { GameState() },
                { _, v, gameState ->
                    gameState.add(
                       v
                    )
                    gameState
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
            .map { k, v ->
                println("\uD83D\uDCBE          ${k?.toString()?.take(8)} AGGRGATE Turn ${v.turn} PlayerUp ${v.playerUp}")
                KeyValue(k,jsonMapper.writeValueAsString(v))
            }.to(
                GAME_STATES_CHANGELOG,
                Produced.with(Serdes.UUID(), Serdes.String())
            )


        gameStates
            .toStream()
            .filter { _, v ->
                v.moves.isNotEmpty()
            }
            .mapValues { v -> v.moves.last() }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(MOVE_MADE_EV,
                Produced.with(Serdes.UUID(), Serdes.String()))

        val topology = streamsBuilder.build()

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-gamestates-aggregator"
        props["processing.guarantee"] = "exactly_once"

        waitForTopics(Topics.all, props)

        val streams = KafkaStreams(topology, props)
        streams.start()
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
