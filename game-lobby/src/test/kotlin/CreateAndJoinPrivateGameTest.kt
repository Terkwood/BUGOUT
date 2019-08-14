
import org.apache.kafka.streams.TopologyTestDriver
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

    // TODO


    @AfterAll
    fun tearDown() {
        testDriver.close()
    }

}