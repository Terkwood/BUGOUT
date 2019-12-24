import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.*
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class CreateGameTest {
    private val testDriver: TopologyTestDriver = setup()

    @BeforeAll
    fun init() {
        initLobby(testDriver)
    }


    @Test
    fun createGameStreamsToGameLobby() {

        val expectedGames = mutableListOf<Game>()
        listOf(Visibility.Public, Visibility.Private).forEach { v ->
            val factory =
                ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

            val creatorClientId = UUID.randomUUID()
            val creatorSessionId = UUID.randomUUID()

            val newGameId = UUID.randomUUID()
            val cgReq = CreateGame(
                clientId = creatorClientId,
                visibility = v,
                gameId = newGameId,
                sessionId = creatorSessionId
            )

            testDriver.pipeInput(
                factory.create(
                    Topics.CREATE_GAME,
                    creatorSessionId,
                    jsonMapper.writeValueAsString(cgReq)
                )
            )


            val waitOutput =
                testDriver.readOutput(
                    Topics.WAIT_FOR_OPPONENT,
                    UUIDDeserializer(),
                    StringDeserializer()
                )

            val actualWait = jsonMapper.readValue(
                waitOutput.value(), WaitForOpponent::class
                    .java
            )

            OutputVerifier.compareKeyValue(
                waitOutput,
                creatorSessionId,
                jsonMapper.writeValueAsString(
                    WaitForOpponent
                        (
                        gameId = newGameId,
                        sessionId = creatorSessionId,
                        eventId =
                        actualWait.eventId,
                        visibility = v
                    )
                )
            )

            val gameLobbyCommandOutput =
                testDriver.readOutput(
                    Topics.GAME_LOBBY_COMMANDS,
                    StringDeserializer(),
                    StringDeserializer()
                )


            OutputVerifier.compareKeyValue(
                gameLobbyCommandOutput,
                GameLobby.TRIVIAL_KEY,
                jsonMapper.writeValueAsString(
                    GameLobbyCommand(
                        Game(newGameId, v, creator = creatorSessionId),
                        LobbyCommand.Open
                    )
                )
            )

            val gameStatesChangelogOutput =
                testDriver.readOutput(
                    Topics.GAME_LOBBY_CHANGELOG,
                    StringDeserializer(), StringDeserializer()
                )

            val expectedLobby = GameLobby()

            expectedGames += Game(newGameId, v, creatorSessionId)
            expectedLobby.games = expectedGames

            OutputVerifier.compareKeyValue(
                gameStatesChangelogOutput,
                GameLobby.TRIVIAL_KEY,
                jsonMapper.writeValueAsString(expectedLobby)
            )
        }


    }


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }

}