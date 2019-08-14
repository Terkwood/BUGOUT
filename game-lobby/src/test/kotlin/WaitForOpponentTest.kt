import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.*
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class WaitForOpponentTest {
    private val testDriver: TopologyTestDriver = setup()

    @BeforeAll
    fun init() {
        initLobby(testDriver)
    }


    @Test
    fun createGameRequestGeneratesWaitForOpponentEvent() {
        listOf(Visibility.Public, Visibility.Private).forEach { v ->
            val factory =
                ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

            val creatorClientId = UUID.randomUUID()

            val newGameId = UUID.randomUUID()
            val cgReq = CreateGame(
                clientId = creatorClientId,
                visibility = v,
                gameId = newGameId
            )

            testDriver.pipeInput(
                factory.create(
                    Topics.CREATE_GAME,
                    creatorClientId,
                    jsonMapper.writeValueAsString(cgReq)
                )
            )

            val output =
                testDriver.readOutput(
                    Topics.WAIT_FOR_OPPONENT,
                    UUIDDeserializer(), StringDeserializer()
                )

            val actual = jsonMapper.readValue(
                output.value(), WaitForOpponent::class
                    .java
            )

            OutputVerifier.compareKeyValue(
                output,
                creatorClientId,
                jsonMapper.writeValueAsString(
                    WaitForOpponent
                        (
                        gameId = newGameId,
                        clientId = creatorClientId,
                        eventId =
                        actual.eventId
                    )
                )
            )
        }
    }


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }


}