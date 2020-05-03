import java.util.Properties
import java.util.UUID
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TestInputTopic
import org.apache.kafka.streams.TopologyTestDriver
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.TestInstance

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class TopologyTest {
    private val testDriver: TopologyTestDriver = setup()

    @Test
    fun testNoOpReq() {
        val turn = 3
        val playerUp = TODO()
        val moves = TODO()

        val gameId = UUID.randomUUID()
        val gameStateJson: String = TODO()

        val gameStatesIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(Topics.GAME_STATES_CHANGELOG,
                        TODO("uuidSerde"), TODO("stringSerde"))
        gameStatesIn.pipeInput(gameId, gameStateJson)

        val reqSyncJson: String = TODO()
        val sessionId = UUID.randomUUID()
        val rsReqId = UUID.randomUUID()
        val lastMove = TODO()

        val reqSyncIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(Topics.REQ_SYNC_CMD,
                        TODO("uuidSerde"), TODO("stringSerde"))
        reqSyncIn.pipeInput(sessionId, reqSyncJson)

        TODO()
    }
}

fun setup(): TopologyTestDriver {
    val props = Properties()
    props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-sync"
    props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = "dummy:1234"
    props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"
    return TopologyTestDriver(Application("dummy-brokers").build(), props)
}
