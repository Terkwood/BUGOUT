import org.apache.kafka.common.serialization.*
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*


@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class DuplicatedGameStateSanityTest {
    private val testDriver: TopologyTestDriver = setup()

    @Test
    fun bogus() {
        assert(false)
    }

    @AfterAll
    fun tearDown() {
        testDriver.close()
    }
}

fun setup(): TopologyTestDriver {
    // setup test driver
    val props = Properties()
    props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = "no-matter"
    props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-changelog"
    props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

    val topo = Aggregator("dummy-brokers").build()
    println(topo.describe())
    return TopologyTestDriver(topo, props)
}