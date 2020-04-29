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
class GameTest {
    private val testDriver: TopologyTestDriver = setup()

    @Test
    fun basicTest() {

        val gameId = UUID.randomUUID()
        val replyTo = UUID.randomUUID()
        val eventId = UUID.randomUUID()
        val player = Player.BLACK
        val coord = Coord(4,4)

        val gameReady = GameReady()

        val firstMove = MoveMade(  gameId, replyTo, eventId, player, coord, listOf())

        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        testDriver.pipeInput(
            factory.create(Topics.GAME_READY,
                gameId,
                jsonMapper.writeValueAsString(gameReady)))


        testDriver.pipeInput(
            factory.create(
                Topics.MOVE_ACCEPTED_EV,
                gameId,
                jsonMapper.writeValueAsString(firstMove)
            )
        )



        val firstOutputRecord =
            testDriver.readOutput(
                Topics.MOVE_MADE_EV,
                UUIDDeserializer(),
                StringDeserializer()
            )



        val actualFirst: MoveMade =
            jsonMapper.readValue(firstOutputRecord.value(), MoveMade::class.java)

        val expectedFirst =
            jsonMapper.writeValueAsString(firstMove)

        OutputVerifier.compareKeyValue(firstOutputRecord, actualFirst.gameId, expectedFirst)


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

        val secondMove = MoveMade(gameId, UUID.randomUUID(),UUID.randomUUID(),Player.WHITE, Coord(10,10))

        testDriver.pipeInput(
            factory.create(
                Topics.MOVE_ACCEPTED_EV,
                gameId,
                jsonMapper.writeValueAsString(secondMove)
            )
        )


        val secondOutputRecord =
            testDriver.readOutput(
                Topics.MOVE_MADE_EV,
                UUIDDeserializer(),
                StringDeserializer()
            )



        val actualSecond: MoveMade =
            jsonMapper.readValue(secondOutputRecord.value(), MoveMade::class.java)

        val expectedSecond =
            jsonMapper.writeValueAsString(secondMove)

        OutputVerifier.compareKeyValue(secondOutputRecord, actualSecond.gameId, expectedSecond)

        testDriver.readOutput(
            Topics.GAME_STATES_CHANGELOG,
            UUIDDeserializer(),
            StringDeserializer()
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