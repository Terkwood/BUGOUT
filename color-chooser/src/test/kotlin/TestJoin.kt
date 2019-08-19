import org.apache.kafka.common.serialization.*
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class TestJoin {
    private val testDriver: TopologyTestDriver = setup()


    @Test
    fun testJoin() {

        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val clientOnePref = ChooseColorPref(clientOne, ColorPref.Black)
        val clientTwoPref = ChooseColorPref(clientTwo, ColorPref.White)

        val gameReadyEvent = GameReady(gameId, Pair(clientOne, clientTwo), eventId = UUID.randomUUID())

        val factory =
            ConsumerRecordFactory(
                UUIDSerializer(), StringSerializer()
            )


        testDriver.pipeInput(
            factory.create(
                Topics.CHOOSE_COLOR_PREF,
                clientOne,
                jsonMapper.writeValueAsString(clientOnePref)
            )
        )

        testDriver.pipeInput(
            factory.create(
                Topics.CHOOSE_COLOR_PREF,
                clientTwo,
                jsonMapper.writeValueAsString(clientTwoPref)
            )
        )


        testDriver.pipeInput(
            factory.create(
                Topics.GAME_READY,
                gameId,
                jsonMapper.writeValueAsString(gameReadyEvent)
            )
        )


        val gcpOne =
            testDriver.readOutput(
                Topics.GAME_COLOR_PREF,
                UUIDDeserializer(),
                StringDeserializer()
            )


        val gcpTwo =
            testDriver.readOutput(
                Topics.GAME_COLOR_PREF,
                UUIDDeserializer(),
                StringDeserializer()
            )


        val gameColorPrefs =
            listOf(gcpOne.value(), gcpTwo.value())
                .map { jsonMapper.readValue(it, GameColorPref::class.java) }

        val expectedSize = 2
        Assertions.assertEquals(expectedSize,
            gameColorPrefs.size)

        Assertions.assertArrayEquals(
            listOf(
                GameColorPref(clientOne,gameId,clientOnePref.colorPref),
                GameColorPref(clientTwo,gameId,clientTwoPref.colorPref)
            ).toTypedArray(),
            gameColorPrefs.toTypedArray())
    }

    @AfterAll
    fun tearDown() {
        testDriver.close()
    }
}