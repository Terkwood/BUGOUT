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

    private fun push(
        c1: ChooseColorPref,
        c2: ChooseColorPref,
        gameId: GameId
    ): ProducerRecord<UUID, String>? {

        val gameReadyEvent = GameReady(
            gameId,
            Pair(c1.clientId, c2.clientId),
            eventId = UUID.randomUUID()
        )

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
        val sessionOne = UUID.randomUUID()
        val sessionTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val c1Pref = ChooseColorPref(clientOne, ColorPref.White,sessionOne)
        val c2Pref = ChooseColorPref(clientTwo, ColorPref.Black,sessionTwo)

        val chosen = push(
            c1Pref,
            c2Pref, gameId
        )


        OutputVerifier.compareKeyValue(
            chosen, gameId,
            jsonMapper.writeValueAsString(
                ColorsChosen(
                    gameId = gameId,
                    black = clientTwo,
                    white = clientOne
                )
            )
        )
    }

    @Test
    fun testAnotherNoConflict() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val sessionOne = UUID.randomUUID()
        val sessionTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen = push(
            ChooseColorPref(clientOne, ColorPref.Black,sessionOne),
            ChooseColorPref(clientTwo, ColorPref.White,sessionTwo), gameId
        )

        OutputVerifier.compareKeyValue(
            chosen, gameId,
            jsonMapper.writeValueAsString(
                ColorsChosen(
                    gameId = gameId,
                    black = clientOne,
                    white = clientTwo
                )
            )
        )
    }

    @Test
    fun testConflict() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val sessionOne = UUID.randomUUID()
        val sessionTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen: ColorsChosen = jsonMapper.readValue(push(
            ChooseColorPref(clientOne, ColorPref.Black,sessionOne),
            ChooseColorPref(clientTwo, ColorPref.Black,sessionTwo), gameId
        )?.value(), ColorsChosen::class.java)

        Assertions.assertTrue(when (chosen.black) {
            clientOne -> chosen.white == clientTwo
            else -> chosen.black == clientTwo && chosen.white == clientOne
        })
    }

    @Test
    fun testMoreConflict() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val sessionOne = UUID.randomUUID()
        val sessionTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen: ColorsChosen = jsonMapper.readValue(push(
            ChooseColorPref(clientOne, ColorPref.White,sessionOne),
            ChooseColorPref(clientTwo, ColorPref.White,sessionTwo), gameId
        )?.value(), ColorsChosen::class.java)

        Assertions.assertTrue(when (chosen.black) {
            clientOne -> chosen.white == clientTwo
            else -> chosen.black == clientTwo && chosen.white == clientOne
        })
    }

    @Test
    fun testSimpleDemands() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val sessionOne = UUID.randomUUID()
        val sessionTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen = push(
            ChooseColorPref(clientOne, ColorPref.Any,sessionOne),
            ChooseColorPref(clientTwo, ColorPref.White,sessionTwo), gameId
        )

        OutputVerifier.compareKeyValue(
            chosen, gameId,
            jsonMapper.writeValueAsString(
                ColorsChosen(
                    gameId = gameId,
                    black = clientOne,
                    white = clientTwo
                )
            )
        )
    }

    @Test
    fun testMoreDemands() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val sessionOne = UUID.randomUUID()
        val sessionTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosen = push(
            ChooseColorPref(clientOne, ColorPref.White,sessionOne),
            ChooseColorPref(clientTwo, ColorPref.Any,sessionTwo), gameId
        )

        OutputVerifier.compareKeyValue(
            chosen, gameId,
            jsonMapper.writeValueAsString(
                ColorsChosen(
                    gameId = gameId,
                    black = clientTwo,
                    white = clientOne
                )
            )
        )
    }

    @Test
    fun testLooseConcerns() {
        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val sessionOne = UUID.randomUUID()
        val sessionTwo = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val chosenJson = push(
            ChooseColorPref(clientOne, ColorPref.Any,sessionOne),
            ChooseColorPref(clientTwo, ColorPref.Any,sessionTwo), gameId
        )

        val chosen: ColorsChosen =
            jsonMapper.readValue(chosenJson?.value(), ColorsChosen::class.java)

        when (chosen.black) {
            clientOne -> Assertions.assertTrue(chosen.white == clientTwo)
            else -> Assertions.assertTrue(
                chosen.black == clientTwo &&
                        chosen.white == clientOne
            )
        }

    }


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }
}