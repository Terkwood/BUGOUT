import org.apache.kafka.common.serialization.StringDeserializer
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDDeserializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.lang.NullPointerException
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

        val gameReady = GameReady()

        val mvAccepted = MoveMade(  gameId, replyTo, eventId, player, coord, listOf())

        val beginningState = GameState()

        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        testDriver.pipeInput(
            factory.create(Topics.GAME_READY,
                gameId,
                jsonMapper.writeValueAsString(gameReady)))

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



        val outputRecord =
            testDriver.readOutput(
                Topics.MOVE_MADE_EV,
                UUIDDeserializer(),
                StringDeserializer()
            )



        val actual: MoveMade =
            jsonMapper.readValue(outputRecord.value(), MoveMade::class.java)

        val expected =
            jsonMapper.writeValueAsString(mvAccepted)

        OutputVerifier.compareKeyValue(outputRecord, actual.gameId, expected)


        val possiblyRepeatedOutput =
            testDriver.readOutput(
                Topics.MOVE_MADE_EV,
                UUIDDeserializer(),
                StringDeserializer()
            )

        try {
            jsonMapper.readValue(possiblyRepeatedOutput.value(), MoveMade::class.java)

            assert(false)
        } catch(e: NullPointerException) {
            assert(true)
        }

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