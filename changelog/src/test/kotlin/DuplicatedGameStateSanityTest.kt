import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*


@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class DuplicatedGameStateSanityTest {
    private val testDriver: TopologyTestDriver = setup()

    @Test
    fun duplicateGameStatesCannotTriggerDuplicateMoveMadeEvents() {

        val gameId = UUID.randomUUID()
        val replyTo = UUID.randomUUID()
        val eventId = UUID.randomUUID()
        val player = Player.BLACK
        val coord = Coord(4,4)

        val mvAccepted = MoveMade(  gameId, replyTo, eventId, player, coord, listOf())

        val beginningState = GameState()

        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        testDriver.pipeInput(
            factory.create(
                Topics.GAME_STATES_CHANGELOG,
                gameId,
                jsonMapper.writeValueAsString(beginningState)
            )
        )

        testDriver.pipeInput(
            factory.create(
                Topics.MOVE_ACCEPTED_EV,
                gameId,
                jsonMapper.writeValueAsString(mvAccepted)
            )
        )

        // Dangerously repeat!
        testDriver.pipeInput(
            factory.create(
                Topics.GAME_STATES_CHANGELOG,
                gameId,
                jsonMapper.writeValueAsString(beginningState)
            )
        )


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