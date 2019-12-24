import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.StringDeserializer
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import serdes.jsonMapper
import java.util.*
import org.junit.jupiter.api.*


@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class ClientDisconnectedTest {
    private val testDriver: TopologyTestDriver = setup()

    @BeforeAll
    fun init() {
        initLobby(testDriver)
    }


    @Test
    fun clientDisconnected() {

        val clientId = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val stringKeyFactory =
            ConsumerRecordFactory(StringSerializer(), StringSerializer())

        val lobbyWithOneGame = GameLobby()
        lobbyWithOneGame.games =  listOf(Game(gameId, Visibility
            .Public,
            clientId))
        val cr: ConsumerRecord<ByteArray, ByteArray> =
            stringKeyFactory.create(
                Topics.GAME_LOBBY_CHANGELOG,
                GameLobby.TRIVIAL_KEY,
                jsonMapper.writeValueAsString(lobbyWithOneGame)
            )

        testDriver.pipeInput(cr)


        val uuidKeyFactory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        val disconnectEv = SessionDisconnected(
            sessionId = clientId
        )

        val dcr : ConsumerRecord<ByteArray, ByteArray> =
            uuidKeyFactory.create(
                Topics.CLIENT_DISCONNECTED,
                clientId,
                jsonMapper.writeValueAsString(disconnectEv)
            )
        testDriver.pipeInput(dcr)


        val gameLobbyOutput =
            testDriver.readOutput(
                Topics.GAME_LOBBY_CHANGELOG,
                StringDeserializer(),
                StringDeserializer()
            )


        val expectedGameLobby = GameLobby()

        OutputVerifier.compareKeyValue(
            gameLobbyOutput,
            GameLobby.TRIVIAL_KEY,
            jsonMapper.writeValueAsString(
                expectedGameLobby
            )
        )
    }

    @AfterAll
    fun tearDown() {
        testDriver.close()
    }
}
