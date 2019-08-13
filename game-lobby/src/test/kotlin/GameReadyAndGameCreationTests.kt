import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.StringDeserializer
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDDeserializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class GameReadyAndGameCreationTests {

    private val testDriver: TopologyTestDriver = setup()

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
                AllOpenGames.TRIVIAL_KEY, emptyAgg
            )

        testDriver.pipeInput(cr)
    }


    @Test
    fun emptyGameStatesTriggerGameReady() {
        val factory =
            ConsumerRecordFactory(
                Topics.GAME_STATES_CHANGELOG,
                UUIDSerializer(),
                StringSerializer()
            )

        val gameId = UUID.randomUUID()
        val emptyBoard =
            "{\"board\":{\"pieces\":{},\"size\":19}," +
                    "\"captures\":{\"black\":0,\"white\":0}," +
                    "\"turn\":1," +
                    "\"playerUp\":\"BLACK\"}"
        testDriver.pipeInput(factory.create(gameId, emptyBoard))

        val outputRecord =
            testDriver.readOutput(
                Topics.GAME_READY,
                UUIDDeserializer(),
                StringDeserializer()
            )

        val actual: GameReady = jsonMapper.readValue(outputRecord.value(), GameReady::class.java)
        val expected =
            jsonMapper.writeValueAsString(GameReady(gameId = gameId, eventId = actual.eventId))

        OutputVerifier.compareKeyValue(outputRecord, gameId, expected)
    }


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
    fun createGameStreamsToGameLobbyCommands() {
        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        val reqId = UUID.randomUUID()
        val visibility = Visibility.Public
        val cgReq = CreateGame(reqId = reqId, visibility = visibility)

        testDriver.pipeInput(
            factory.create(
                Topics.CREATE_GAME, reqId, jsonMapper.writeValueAsString(cgReq)
            )
        )

        val outputRecord =
            testDriver.readOutput(
                Topics.GAME_LOBBY_COMMANDS,
                StringDeserializer(),
                StringDeserializer()
            )

        val newGameId = jsonMapper
            .readValue(outputRecord.value(), GameCommand::class.java)
            .game
            .gameId

        OutputVerifier.compareKeyValue(
            outputRecord,
            AllOpenGames.TRIVIAL_KEY,
            jsonMapper.writeValueAsString(GameCommand(Game(newGameId, visibility),
                Command.Open))
        )
    }


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }

}