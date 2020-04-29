import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.junit.jupiter.api.Test
import java.util.*

class NullGameStateTest {
    @Test
    fun guardAgainstNullGameState() {

    }
}

fun setup(): TopologyTestDriver {
    // setup test driver
    val props = Properties()
    props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = "no-matter"
    props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-judge"
    props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

    val topo = Judge("dummy-brokers").build()
    println(topo.describe())
    return TopologyTestDriver(topo, props)
}