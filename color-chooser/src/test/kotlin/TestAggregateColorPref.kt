import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.*
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class TestAggregateColorPref {
    private val testDriver: TopologyTestDriver = setup()

    @BeforeAll
    fun init() {
    }

    @Test
    fun test() {

        val clientOne = UUID.randomUUID()
        val clientTwo = UUID.randomUUID()
        val gameId =UUID.randomUUID()

        val clientOnePref = ChooseColorPref(clientOne, ColorPref.Any)
        val clientTwoPref = ChooseColorPref(clientTwo, ColorPref.Black)

        val gameReadyEvent = GameReady(gameId, Pair(clientOne, clientTwo), eventId=UUID.randomUUID())

        val factory =
            ConsumerRecordFactory(
                UUIDSerializer(), StringSerializer()
            )
        val cr1: ConsumerRecord<ByteArray, ByteArray> = factory.create(
            Topics.CHOOSE_COLOR_PREF,
            clientOne,
            jsonMapper.writeValueAsString(clientOnePref)
        )


        testDriver.pipeInput(cr1)

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



    }

    @AfterAll
    fun tearDown() {
        testDriver.close()
    }
}