import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import java.util.*

class GameLobbyTest {

    fun setup(): TopologyTestDriver {
        // setup test driver
        val props = Properties()
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-game-lobby"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        return TopologyTestDriver(GameLobby("dummy-brokers").build(), props)
    }

    @Test
    fun emptyGameStatesTriggerGameReady() {
        val testDriver = setup()

        val factory =
            ConsumerRecordFactory(
                Topics.GAME_STATES_CHANGELOG,
                UUIDSerializer(),
                StringSerializer()
            )

        val gameId = UUID.randomUUID()
        val emptyBoard =
            "{\"board\":{\"pieces\":{},\"size\":19}," +
                    "\"captures\":{\"black\":0,\"white\":0},\"turn\":1," +
                    "\"playerUp\":\"BLACK\"}"
        testDriver.pipeInput(factory.create(gameId, emptyBoard))

        assertEquals(true, false)
    }
}