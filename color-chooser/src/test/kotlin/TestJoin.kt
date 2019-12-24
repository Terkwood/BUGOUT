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
        val sessionOne = UUID.randomUUID()
        val sessionTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val sessionOnePref = ChooseColorPref(clientOne, ColorPref.Black,sessionOne)
        val sessionTwoPref = ChooseColorPref(clientTwo, ColorPref.White,sessionTwo)

        val gameReadyEvent = GameReady(gameId, Pair(sessionOne, sessionTwo), eventId =
        UUID.randomUUID())

        val factory =
            ConsumerRecordFactory(
                UUIDSerializer(), StringSerializer()
            )


        testDriver.pipeInput(
            factory.create(
                Topics.CHOOSE_COLOR_PREF,
                sessionOne,
                jsonMapper.writeValueAsString(sessionOnePref)
            )
        )

        testDriver.pipeInput(
            factory.create(
                Topics.CHOOSE_COLOR_PREF,
                sessionTwo,
                jsonMapper.writeValueAsString(sessionTwoPref)
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
                GameColorPref(sessionOne,gameId,sessionOnePref.colorPref),
                GameColorPref(sessionTwo,gameId,sessionTwoPref.colorPref)
            ).toTypedArray(),
            gameColorPrefs.toTypedArray())
    }

    @AfterAll
    fun tearDown() {
        testDriver.close()
    }
}