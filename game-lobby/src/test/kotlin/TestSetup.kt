import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import java.util.*

const val EMPTY_LOBBY = "{\"games\":[]}"

fun setup(): TopologyTestDriver {
    // setup test driver
    val props = Properties()
    props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = "no-matter"
    props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-game-lobby"
    props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

    val topo = Application("dummy-brokers").build()
    println(topo.describe())
    return TopologyTestDriver(topo, props)
}

fun initLobby(testDriver: TopologyTestDriver) {
    val factory =
        ConsumerRecordFactory(
            StringSerializer(),
            StringSerializer()
        )


    val cr: ConsumerRecord<ByteArray, ByteArray> =
        factory.create(
            Topics.GAME_LOBBY_CHANGELOG,
            GameLobby.TRIVIAL_KEY, EMPTY_LOBBY
        )

    testDriver.pipeInput(cr)
}