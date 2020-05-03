
import java.util.Properties
import java.util.TimeZone
import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.Topology

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
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "bugout-sync"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        val streams = KafkaStreams(topology, props)

        waitForTopics(Topics.all, props)

        streams.start()
    }

    fun build(): Topology {
        TODO("in the event that the service layer needs to catch up with the " +
                "client's last move, we _must not_ emit the sync reply until " +
                "we hear the move confirmed on bugout-move-made-ev")

        TODO()
    }

    private fun waitForTopics(
        topics: Array<String>,
        props:
            Properties
    ) {
        print("⏲ Waiting for topics ")
        val client = AdminClient.create(props)

        var topicsReady = false
        while (!topicsReady) {
            val found = client.listTopics().names().get()

            val diff = topics.subtract(found.filterNotNull())

            topicsReady = diff.isEmpty()

            if (!topicsReady) Thread.sleep(333)
            print(".")
        }

        println(" done! 🏁")
    }
}
