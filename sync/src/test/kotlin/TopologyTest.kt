import java.util.Properties
import java.util.UUID
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TestInputTopic
import org.apache.kafka.streams.TopologyTestDriver
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.TestInstance
import serdes.jsonMapper

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class TopologyTest {
    private val testDriver: TopologyTestDriver = setup()

    @Test
    fun testNoOpReq() {
        val turn = 3
        val playerUp = Player.BLACK
        val moves = listOf(
                Move(Player.BLACK, Coord(4, 4), 1),
                Move(Player.WHITE, Coord(10, 10), 2)
        )
        val gameId = UUID.randomUUID()
        val reqId = UUID.randomUUID()
        val sessionId = UUID.randomUUID()

        // gateway drives a request
        val lastMove = moves[moves.size - 1]
        val reqSync = ReqSyncCmd(sessionId, reqId, gameId, playerUp, turn, lastMove)

        val reqSyncIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(Topics.REQ_SYNC_CMD,
                        TODO("uuidSerde"), TODO("stringSerde"))
        reqSyncIn.pipeInput(sessionId, jsonMapper.writeValueAsString(reqSync))

        // this is the response we would expect from history provider.
        // sync service will consume this to complete its reply
        val historyProvided = HistoryProvided(gameId, reqId, UUID.randomUUID(), moves)

        val historyProvidedIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(Topics.HISTORY_PROVIDED_EV,
                        TODO("uuidSerde"), TODO("stringSerde"))
        historyProvidedIn.pipeInput(gameId, jsonMapper.writeValueAsString(historyProvided))

        // check to make sure that sync service outputs
        // a reply that won't require the client to do anything
        TODO()
    }
}

fun setup(): TopologyTestDriver {
    val props = Properties()
    props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-sync"
    props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = "dummy:1234"
    props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"
    return TopologyTestDriver(Application("dummy-brokers").build(), props)
}
