import org.apache.kafka.clients.consumer.ConsumerRecord
import org.apache.kafka.common.serialization.StringDeserializer
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDDeserializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.AfterAll
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.TestInstance
import java.util.*

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
@Disabled
class GameLobbyTest {

    private val testDriver: TopologyTestDriver = setup()

    private fun setup(): TopologyTestDriver {
        // setup test driver
        val props = Properties()
        props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = "no-matter"
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-game-lobby"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        return TopologyTestDriver(GameLobby("dummy-brokers").build(), props)
    }

    @Test
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
                    "\"captures\":{\"black\":0,\"white\":0},\"turn\":1," +
                    "\"playerUp\":\"BLACK\"}"
        testDriver.pipeInput(factory.create(gameId, emptyBoard))

        val outputRecord =
            testDriver.readOutput(
                Topics.GAME_READY,
                UUIDDeserializer(),
                StringDeserializer()
            )

        OutputVerifier.compareKeyValue(outputRecord, gameId, "bad idea")
    }

    @AfterAll
    fun tearDown() {
        testDriver.close()
    }

}