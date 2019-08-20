import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import java.util.*

fun setup(): TopologyTestDriver {
    // setup test driver
    val props = Properties()
    props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = "no-matter"
    props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-color-chooser"
    props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"


    val topology = Application("dummy-brokers").build()
    println(topology.describe())
    return TopologyTestDriver(topology, props)
}