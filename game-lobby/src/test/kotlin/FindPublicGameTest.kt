import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.*
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.*
import serdes.jsonMapper
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class FindPublicGameTest {

    private val testDriver: TopologyTestDriver = setup()

    @BeforeAll
    fun init() {
        initLobby(testDriver)
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
        val sessions = Pair(UUID.randomUUID(), UUID.randomUUID())
        val fpg = {
                client: ClientId, session: SessionId ->
            FindPublicGame(client, session) }

        val f1: ConsumerRecord<ByteArray, ByteArray> =
            fpgFactory.create(
                Topics.FIND_PUBLIC_GAME,
                sessions.first, jsonMapper.writeValueAsString(
                    fpg(
                        clients.first,
                        sessions.first
                    )
                )
            )
        val f2: ConsumerRecord<ByteArray, ByteArray> =
            fpgFactory.create(
                Topics.FIND_PUBLIC_GAME,
                sessions.second, jsonMapper.writeValueAsString(
                    fpg(
                        clients.second, sessions.second
                    )
                )
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
                    sessions = sessions
                )
            )

        OutputVerifier.compareKeyValue(outputRecord, actual.gameId, expected)
    }


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }

}