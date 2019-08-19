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

    // hacks to make serialization work
    val strings = Serdes.String()
    val uuids = Serdes.UUID()
    props[StreamsConfig.DEFAULT_KEY_SERDE_CLASS_CONFIG] = uuids::class.java.name
    props[StreamsConfig.DEFAULT_VALUE_SERDE_CLASS_CONFIG] = strings::class.java.name


    val topology = Application("dummy-brokers").build()
    println(topology.describe())
    return TopologyTestDriver(topology, props)
}