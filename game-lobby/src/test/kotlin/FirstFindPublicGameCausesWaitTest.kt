import org.apache.kafka.common.serialization.*
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class FirstFindPublicGameCausesWaitTest {
    private val testDriver: TopologyTestDriver = setup()

    @BeforeAll
    fun init() {
        initLobby(testDriver)
    }


    @Test
    fun firstFindPublicGameCausesWaitForOpponentEvent() {
        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        val creatorClientId = UUID.randomUUID()
        val creatorSessionId = UUID.randomUUID()

        val fpg = FindPublicGame(
            clientId = creatorClientId, sessionId = creatorSessionId
        )

        testDriver.pipeInput(
            factory.create(
                Topics.FIND_PUBLIC_GAME,
                creatorSessionId,
                jsonMapper.writeValueAsString(fpg)
            )
        )

        val output =
            testDriver.readOutput(
                Topics.WAIT_FOR_OPPONENT,
                UUIDDeserializer(),
                StringDeserializer()
            )

        val actual = jsonMapper.readValue(
            output.value(), WaitForOpponent::class
                .java
        )

        OutputVerifier.compareKeyValue(
            output,
            creatorSessionId,
            jsonMapper.writeValueAsString(
                WaitForOpponent
                    (
                    gameId = actual.gameId,
                    sessionId = creatorSessionId,
                    eventId = actual.eventId,
                    visibility = Visibility.Public
                )
            )
        )
    }



    @AfterAll
    fun tearDown() {
        testDriver.close()
    }


}