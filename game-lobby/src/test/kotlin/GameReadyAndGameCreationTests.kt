import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.*
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



    @Test
    fun twoPeopleCanJoinAPublicGame() {


        // Starting from a fresh kafka cluster, we need two FindPublicGame
        // events to trigger a game ready event

        val fpgFactory =
            ConsumerRecordFactory(
                UUIDSerializer(),
                StringSerializer()
            )
        val clients = Pair(UUID.randomUUID(), UUID.randomUUID())
        val fpg = { client: ClientId -> FindPublicGame(client)}

        val f1: ConsumerRecord<ByteArray, ByteArray> =
            fpgFactory.create(
                Topics.FIND_PUBLIC_GAME,
                clients.first, jsonMapper.writeValueAsString(fpg(clients
                    .first))
            )
        val f2: ConsumerRecord<ByteArray, ByteArray> =
            fpgFactory.create(
                Topics.FIND_PUBLIC_GAME,
                clients.second, jsonMapper.writeValueAsString(fpg(clients
                    .second))
            )

        testDriver.pipeInput(f1)

        testDriver.pipeInput(f2)

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
                    gameId = actual.gameId,
                    eventId = actual.eventId,
                    clients = clients
                )
            )

        OutputVerifier.compareKeyValue(outputRecord, actual.gameId, expected)
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