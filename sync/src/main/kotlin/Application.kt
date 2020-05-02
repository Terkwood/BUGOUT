import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.Topology
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

    fun build() : Topology {
        TODO()
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