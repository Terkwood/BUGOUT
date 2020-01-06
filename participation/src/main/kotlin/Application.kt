import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.*
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import serdes.jsonMapper
import java.util.*

const val BROKERS = "kafka:9092"

fun main() {
    TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
    Application(BROKERS).process()
}

class Application(private val brokers: String) {
    fun process() {
        val topology = build()

        println(topology.describe())

        val props = Properties()
        props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = brokers
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "bugout-participation"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        val streams = KafkaStreams(topology, props)

        waitForTopics(Topics.all, props)

        streams.start()
    }

    fun build(): Topology {
        val streamsBuilder = StreamsBuilder()

        val gameReady: KStream<GameId, GameReady> =
            streamsBuilder.stream<GameId, String>(
                Topics.GAME_READY, Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues {
                    v -> jsonMapper.readValue(v, GameReady::class.java)
            }

        gameReady.mapValues {
            v ->
            {
                println("game ready $v")
                v
            }
        }


        return streamsBuilder.build()
    }

    private fun waitForTopics(topics: Array<String>, props:
    Properties) {
        print("⏲ Waiting for topics ")
        val client = AdminClient.create(props)

        var topicsReady = false
        while(!topicsReady) {
            val found = client.listTopics().names().get()

            val diff = topics.subtract(found.filterNotNull())

            topicsReady = diff.isEmpty()

            if (!topicsReady) Thread.sleep(333)
            print(".")
        }

        println(" done! 🏁")
    }
}