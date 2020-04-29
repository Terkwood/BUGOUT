import org.apache.kafka.common.serialization.StringDeserializer
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDDeserializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.AfterAll
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.TestInstance
import serdes.jsonMapper
import java.util.*


@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class NullGameStateTest {
    //private val testDriver: TopologyTestDriver = setup()

/*  // doesn't help
    @Test
    fun guardAgainstNullGameState() {
        val gameId = UUID.randomUUID()
        val reqId = UUID.randomUUID()
        val player = Player.BLACK
        val coord = Coord(4, 4)

        val makeMoveCmd = MakeMoveCmd(gameId, reqId, player, coord)

        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        testDriver.pipeInput(factory.create(MAKE_MOVE_CMD_TOPIC,
            gameId,
            jsonMapper.writeValueAsString(makeMoveCmd)))

        testDriver.pipeInput(
            factory.create(GAME_STATES_CHANGELOG_TOPIC,
                gameId,
                jsonMapper.writeValueAsString(GameState())))


        testDriver.pipeInput(factory.create(MAKE_MOVE_CMD_TOPIC,
            gameId,
            jsonMapper.writeValueAsString(makeMoveCmd)))

        val output = testDriver.readOutput(MOVE_ACCEPTED_EV_TOPIC,
            UUIDDeserializer(),
            StringDeserializer())

    }
*/


    /*@AfterAll
    fun tearDown() {
        testDriver.close()
    }*/
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