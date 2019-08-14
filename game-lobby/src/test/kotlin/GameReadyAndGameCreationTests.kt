import org.apache.kafka.clients.consumer.ConsumerRecord
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
class GameReadyAndGameCreationTests {

    private val testDriver: TopologyTestDriver = setup()
    private val emptyBoard =
        "{\"board\":{\"pieces\":{},\"size\":19}," +
                "\"captures\":{\"black\":0,\"white\":0}," +
                "\"turn\":1," +
                "\"playerUp\":\"BLACK\"}"

    @BeforeAll
    fun initializeAggregation() {
        val factory =
            ConsumerRecordFactory(
                StringSerializer(),
                StringSerializer()
            )

        val emptyAgg = "{\"games\":[]}"

        val cr: ConsumerRecord<ByteArray, ByteArray> =
            factory.create(
                Topics.GAME_LOBBY_CHANGELOG,
                GameLobby.TRIVIAL_KEY, emptyAgg
            )

        testDriver.pipeInput(cr)
    }


    // TODO this seems too simplistic...
    // TODO   in total, we need a CreateGame event
    // TODO    _and either_ a JoinPrivateGame   _or_
    // TODO     a FindPublicGame  request   in order
    // TODO  see a GameReady
    /*
    @Test
    fun emptyGameStatesTriggerGameReady() {
        val factory =
            ConsumerRecordFactory(
                Topics.GAME_STATES_CHANGELOG,
                UUIDSerializer(),
                StringSerializer()
            )

        val gameId = UUID.randomUUID()

        testDriver.pipeInput(factory.create(gameId, emptyBoard))

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
                    gameId = gameId,
                    eventId = actual.eventId,
                    clients = throw NotImplementedError()
                )
            )

        OutputVerifier.compareKeyValue(outputRecord, gameId, expected)
    }
    */


    @Test
    fun midGameStatesDoNotTriggerGameReady() {
        val factory =
            ConsumerRecordFactory(
                Topics.GAME_STATES_CHANGELOG,
                UUIDSerializer(),
                StringSerializer()
            )

        val gameId = UUID.randomUUID()

        val turnTwo =
            "{\"board\":{\"pieces\":{},\"size\":19}," +
                    "\"captures\":{\"black\":0,\"white\":0}," +
                    "\"turn\":2," +
                    "\"playerUp\":\"BLACK\"}"
        testDriver.pipeInput(factory.create(gameId, turnTwo))

        Assertions.assertNull(
            testDriver.readOutput(
                Topics.GAME_READY,
                UUIDDeserializer(),
                StringDeserializer()
            )
        )
    }


    @Test
    fun createGameStreamsHappily() {
        val expectedGames = mutableListOf<Game>()
        listOf(Visibility.Public, Visibility.Private).forEach { v ->
            val factory =
                ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

            val creatorClientId = UUID.randomUUID()
            val cgReq = CreateGame(clientId = creatorClientId, visibility = v)

            testDriver.pipeInput(
                factory.create(
                    Topics.CREATE_GAME,
                    creatorClientId,
                    jsonMapper.writeValueAsString(cgReq)
                )
            )

            val gameLobbyCommandOutput =
                testDriver.readOutput(
                    Topics.GAME_LOBBY_COMMANDS,
                    StringDeserializer(),
                    StringDeserializer()
                )

            val newGameId = jsonMapper
                .readValue(
                    gameLobbyCommandOutput.value(),
                    GameLobbyCommand::class.java
                )
                .game
                .gameId

            OutputVerifier.compareKeyValue(
                gameLobbyCommandOutput,
                GameLobby.TRIVIAL_KEY,
                jsonMapper.writeValueAsString(
                    GameLobbyCommand(
                        Game(newGameId, v, creator = creatorClientId),
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
            expectedGames += Game(newGameId, v, creatorClientId)
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