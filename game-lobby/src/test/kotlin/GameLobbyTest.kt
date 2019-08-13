import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import java.util.*

class GameLobbyTest {

    fun setup(): TopologyTestDriver {
        // setup test driver
        val props = Properties()
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-game-lobby"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        return TopologyTestDriver(GameLobby("dummy-brokers").build(), props)
    }
}