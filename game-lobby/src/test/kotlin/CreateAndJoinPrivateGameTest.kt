import org.apache.kafka.common.serialization.StringDeserializer
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDDeserializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class CreateAndJoinPrivateGameTest {
    private val testDriver: TopologyTestDriver = setup()


    @BeforeAll
    fun init() {
        initLobby(testDriver)


    }

    @Test
    fun joinValidGame() {

        val creatorClientId = UUID.randomUUID()
        val creatorSessionId = UUID.randomUUID()
        val validGameId = UUID.randomUUID()
        // create a game with a known ID
        val cgReq = CreateGame(
            clientId = creatorClientId,
            visibility = Visibility.Private,
            gameId = validGameId,
            sessionId = creatorSessionId
        )

        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())


        testDriver.pipeInput(
            factory.create(
                Topics.CREATE_GAME,
                creatorSessionId,
                jsonMapper.writeValueAsString(cgReq)
            )
        )

        // someone else tries to join that game
        val joinerClientId = UUID.randomUUID()
        val joinerSessionId = UUID.randomUUID()
        val joinRequest =
            JoinPrivateGame(
                clientId = joinerClientId,
                gameId = validGameId,
                sessionId = creatorSessionId
            )


        testDriver.pipeInput(
            factory.create(
                Topics.JOIN_PRIVATE_GAME,
                joinerSessionId,
                jsonMapper.writeValueAsString(joinRequest)
            )
        )


        val outputRecord =
            testDriver.readOutput(
                Topics.GAME_READY,
                UUIDDeserializer(),
                StringDeserializer()
            )


        val actual: GameReady =
            jsonMapper.readValue(outputRecord.value(), GameReady::class.java)
        val expected =
            jsonMapper.writeValueAsString(
                GameReady(
                    gameId = validGameId,
                    eventId = actual.eventId,
                    sessions = Pair(creatorSessionId, joinerSessionId)
                )
            )

        OutputVerifier.compareKeyValue(outputRecord, actual.gameId, expected)

    }

    @Test
    fun joinInvalidGame() {

        val creatorClientId = UUID.randomUUID()
        val creatorSessionId = UUID.randomUUID()
        val validGameId = UUID.randomUUID()
        // create a game with a known ID
        val cgReq = CreateGame(
            clientId = creatorClientId,
            visibility = Visibility.Private,
            gameId = validGameId,
            sessionId = creatorSessionId
        )

        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        testDriver.pipeInput(
            factory.create(
                Topics.CREATE_GAME,
                creatorSessionId,
                jsonMapper.writeValueAsString(cgReq)
            )
        )


        // someone else tries to join that game
        val joinerClientId = UUID.randomUUID()
        val joinerSessionId = UUID.randomUUID()
        val bogusGameId = UUID.randomUUID()
        val joinRequest =
            JoinPrivateGame(
                clientId = joinerClientId,
                gameId = bogusGameId,
                sessionId = joinerSessionId
            )


        testDriver.pipeInput(
            factory.create(
                Topics.JOIN_PRIVATE_GAME,
                joinerSessionId,
                jsonMapper.writeValueAsString(joinRequest)
            )
        )


        val outputRecord =
            testDriver.readOutput(
                Topics.PRIVATE_GAME_REJECTED,
                UUIDDeserializer(),
                StringDeserializer()
            )


        val actual: PrivateGameRejected =
            jsonMapper.readValue(
                outputRecord.value(),
                PrivateGameRejected::class.java
            )
        val expected =
            jsonMapper.writeValueAsString(
                PrivateGameRejected(
                    gameId = bogusGameId,
                    eventId = actual.eventId,
                    clientId = joinerClientId,
                    sessionId = joinerSessionId
                )
            )

        OutputVerifier.compareKeyValue(outputRecord, joinerSessionId, expected)

    }


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }

}