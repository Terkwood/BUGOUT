import org.apache.kafka.clients.producer.ProducerRecord
import org.apache.kafka.common.serialization.*
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class TestChoice {
    private val testDriver: TopologyTestDriver = setup()

    @BeforeAll
    fun init() {
    }

    private fun push(c1: ChooseColorPref, c2: ChooseColorPref, gameId:GameId): ProducerRecord<UUID, String>? {

        val gameReadyEvent = GameReady(gameId, Pair(c1.clientId, c2.clientId), eventId = UUID.randomUUID())

        val factory =
            ConsumerRecordFactory(
                UUIDSerializer(), StringSerializer()
            )


        testDriver.pipeInput(
            factory.create(
                Topics.CHOOSE_COLOR_PREF,
                c1.clientId,
                jsonMapper.writeValueAsString(c1)
            )
        )

        testDriver.pipeInput(
            factory.create(
                Topics.CHOOSE_COLOR_PREF,
                c2.clientId,
                jsonMapper.writeValueAsString(c2)
            )
        )


        testDriver.pipeInput(
            factory.create(
                Topics.GAME_READY,
                gameId,
                jsonMapper.writeValueAsString(gameReadyEvent)
            )
        )

        return testDriver.readOutput(
                Topics.COLORS_CHOSEN,
                UUIDDeserializer(),
                StringDeserializer()
            )


    }

    @Test
    fun testNoConflict() {

        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen = push(
            ChooseColorPref(clientOne,ColorPref.White),
            ChooseColorPref(clientTwo,ColorPref.Black), gameId)

        OutputVerifier.compareKeyValue(
            chosen, gameId,
            jsonMapper.writeValueAsString(
                ColorsChosen(gameId = gameId, black = clientTwo, white = clientOne)
            )
        )
    }

    @Test
    fun testAnotherNoConflict() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen = push(
            ChooseColorPref(clientOne,ColorPref.Black),
            ChooseColorPref(clientTwo,ColorPref.White), gameId)

        OutputVerifier.compareKeyValue(
            chosen, gameId,
            jsonMapper.writeValueAsString(
                ColorsChosen(gameId = gameId, black = clientOne, white = clientTwo)
            )
        )
    }


    @Test
    fun testSimpleDemands() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen = push(
            ChooseColorPref(clientOne,ColorPref.Any),
            ChooseColorPref(clientTwo,ColorPref.White), gameId)

        OutputVerifier.compareKeyValue(
            chosen, gameId,
            jsonMapper.writeValueAsString(
                ColorsChosen(gameId = gameId, black = clientOne, white = clientTwo)
            )
        )
    }

    @Test
    fun testMoreDemands() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen = push(
            ChooseColorPref(clientOne,ColorPref.White),
            ChooseColorPref(clientTwo,ColorPref.Any), gameId)

        OutputVerifier.compareKeyValue(
            chosen, gameId,
            jsonMapper.writeValueAsString(
                ColorsChosen(gameId = gameId, black = clientTwo, white = clientOne)
            )
        )
    }

    @Test
    fun testLooseConcerns() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen = push(
            ChooseColorPref(clientOne,ColorPref.Any),
            ChooseColorPref(clientTwo,ColorPref.Any), gameId)

        TODO("output verifier check")
    }


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }
}