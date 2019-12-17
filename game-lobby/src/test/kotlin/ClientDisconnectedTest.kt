import org.apache.kafka.common.serialization.StringDeserializer
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDDeserializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.streams.TopologyTestDriver
import org.apache.kafka.streams.test.ConsumerRecordFactory
import org.apache.kafka.streams.test.OutputVerifier
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.TestInstance
import serdes.jsonMapper
import java.util.*

@Disabled
@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class ClientDisconnectedTest {
    private val testDriver: TopologyTestDriver = setup()

    @BeforeAll
    fun init() {
        initLobby(testDriver)
    }


    @Test
    fun clientDisconnected() {
        val factory =
            ConsumerRecordFactory(UUIDSerializer(), StringSerializer())

        val clientId = UUID.randomUUID()
        val gameId = UUID.randomUUID()

        val disconnectEv = ClientDisconnected(
            clientId = clientId
        )

        testDriver.pipeInput(
            factory.create(
                Topics.CLIENT_DISCONNECTED,
                clientId,
                jsonMapper.writeValueAsString(disconnectEv)
            )
        )


        val gameLobbyOutput =
            testDriver.readOutput(
                Topics.GAME_LOBBY_CHANGELOG,
                UUIDDeserializer(),
                StringDeserializer()
            )

        val actualGameLobby = jsonMapper.readValue(
            gameLobbyOutput.value(), GameLobby::class
                .java
        )

        OutputVerifier.compareKeyValue(
            gameLobbyOutput,
            clientId,
            jsonMapper.writeValueAsString(
                TODO()
            )
        )
    }
}