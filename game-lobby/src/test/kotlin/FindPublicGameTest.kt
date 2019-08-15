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
        val fpg = { client: ClientId -> FindPublicGame(client) }

        val f1: ConsumerRecord<ByteArray, ByteArray> =
            fpgFactory.create(
                Topics.FIND_PUBLIC_GAME,
                clients.first, jsonMapper.writeValueAsString(
                    fpg(
                        clients
                            .first
                    )
                )
            )
        val f2: ConsumerRecord<ByteArray, ByteArray> =
            fpgFactory.create(
                Topics.FIND_PUBLIC_GAME,
                clients.second, jsonMapper.writeValueAsString(
                    fpg(
                        clients
                            .second
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
                    clients = clients
                )
            )

        OutputVerifier.compareKeyValue(outputRecord, actual.gameId, expected)
    }


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }

}