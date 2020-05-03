
import java.util.Properties
import java.util.UUID
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.TestInputTopic
import org.apache.kafka.streams.TestOutputTopic
import org.apache.kafka.streams.TopologyTestDriver
import org.junit.jupiter.api.AfterAll
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.TestInstance
import serdes.jsonMapper

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class TopologyTest {
    private val testDriver: TopologyTestDriver = setup()

    @Test
    fun testNoOpReq() {
        val turn = 3
        val playerUp = Player.BLACK
        val moves = listOf(
                Move(Player.BLACK, Coord(4, 4), 1),
                Move(Player.WHITE, Coord(10, 10), 2)
        )
        val gameId = UUID.randomUUID()
        val reqId = UUID.randomUUID()
        val sessionId = UUID.randomUUID()

        // gateway drives a request
        val lastMove = moves[moves.size - 1]
        val reqSync = ReqSyncCmd(sessionId, reqId, gameId, playerUp, turn,
                lastMove)

        val uuidSerde = Serdes.UUID()
        val stringSerde = Serdes.String()
        val reqSyncIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(
                        Topics.REQ_SYNC_CMD,
                        Serdes.UUID().serializer(),
                        Serdes.String().serializer())
        reqSyncIn.pipeInput(
                sessionId,
                jsonMapper.writeValueAsString(reqSync)
        )

        // this is the response we would expect from history provider.
        // sync service will consume this to complete its reply
        val historyProvided = HistoryProvidedEv(gameId, replyTo = reqId,
                eventId = UUID.randomUUID(), moves = moves)

        val historyProvidedIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(
                        Topics.HISTORY_PROVIDED_EV,
                        uuidSerde.serializer(),
                        stringSerde.serializer())
        historyProvidedIn.pipeInput(gameId,
                jsonMapper.writeValueAsString(historyProvided))

        // check to make sure that sync service outputs
        // a reply that won't require the client to do anything
        val syncReplyOut: TestOutputTopic<UUID, String> =
                testDriver.createOutputTopic(
                        Topics.SYNC_REPLY_EV,
                        uuidSerde.deserializer(),
                        stringSerde.deserializer())

        val expected = SyncReplyEv(
                sessionId = sessionId,
                replyTo = reqId,
                moves = moves,
                gameId = gameId,
                playerUp = playerUp,
                turn = turn
        )

        assertEquals(syncReplyOut.readKeyValue(),
                (KeyValue(sessionId, jsonMapper.writeValueAsString(expected))))
    }

    @Test
    fun testClientCatchUp() {
        val turn = 3
        val playerUp = Player.BLACK
        val moves = listOf(
                Move(Player.BLACK, Coord(4, 4), 1),
                Move(Player.WHITE, Coord(10, 10), 2)
        )
        val gameId = UUID.randomUUID()
        val reqId = UUID.randomUUID()
        val sessionId = UUID.randomUUID()

        // client is behind by one move
        val lastKnownClientMove = moves[0]
        val reqSync = ReqSyncCmd(sessionId = sessionId,
                reqId = reqId,
                gameId = gameId,
                playerUp = Player.WHITE,
                turn = turn - 1,
                lastMove = lastKnownClientMove)

        val uuidSerde = Serdes.UUID()
        val stringSerde = Serdes.String()
        val reqSyncIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(
                        Topics.REQ_SYNC_CMD,
                        Serdes.UUID().serializer(),
                        Serdes.String().serializer())
        reqSyncIn.pipeInput(
                sessionId,
                jsonMapper.writeValueAsString(reqSync)
        )

        // this is the response we would expect from history provider.
        // sync service will consume this to complete its reply
        val historyProvided = HistoryProvidedEv(gameId, replyTo = reqId,
                eventId = UUID.randomUUID(), moves = moves)

        val historyProvidedIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(Topics.HISTORY_PROVIDED_EV,
                        uuidSerde.serializer(),
                        stringSerde.serializer())
        historyProvidedIn.pipeInput(gameId,
                jsonMapper.writeValueAsString(historyProvided))

        // check to make sure that sync service outputs
        // a reply that won't require the client to do anything
        val syncReplyOut: TestOutputTopic<UUID, String> =
                testDriver.createOutputTopic(Topics.SYNC_REPLY_EV,
                        uuidSerde.deserializer(), stringSerde.deserializer())

        val expected = SyncReplyEv(
                sessionId = sessionId,
                replyTo = reqId,
                moves = moves,
                gameId = gameId,
                playerUp = playerUp,
                turn = turn
        )

        assertEquals(syncReplyOut.readKeyValue(),
                (KeyValue(sessionId, jsonMapper.writeValueAsString(expected))))
    }

    @Test
    fun testServerCatchUp() {
        val clientTurn = 4
        val clientPlayerUp = Player.WHITE

        val clientMoves = listOf(
                Move(Player.BLACK, Coord(4, 4), 1),
                Move(Player.WHITE, Coord(10, 10), 2),
                Move(Player.BLACK, Coord(4, 5), 3)
        )
        val serverMoves = clientMoves.subList(0, clientMoves.size - 1)
        val gameId = UUID.randomUUID()
        val reqId = UUID.randomUUID()
        val sessionId = UUID.randomUUID()

        // client sends a request which indicates that
        // server has missed a move
        val clientLastMove = clientMoves[clientMoves.size - 1]
        val reqSync = ReqSyncCmd(sessionId = sessionId,
                reqId = reqId,
                gameId = gameId,
                playerUp = clientPlayerUp,
                turn = clientTurn,
                lastMove = clientLastMove)

        val uuidSerde = Serdes.UUID()
        val stringSerde = Serdes.String()
        val reqSyncIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(
                        Topics.REQ_SYNC_CMD,
                        Serdes.UUID().serializer(),
                        Serdes.String().serializer())
        reqSyncIn.pipeInput(
                sessionId,
                jsonMapper.writeValueAsString(reqSync)
        )

        // this is the response we would expect from history provider.
        // sync service will consume this to complete its reply
        val historyProvided = HistoryProvidedEv(gameId, replyTo = reqId,
                eventId = UUID.randomUUID(), moves = serverMoves)

        val historyProvidedIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(Topics.HISTORY_PROVIDED_EV,
                        uuidSerde.serializer(),
                        stringSerde.serializer())
        historyProvidedIn.pipeInput(gameId,
                jsonMapper.writeValueAsString(historyProvided))

        // check to make sure that we output the client move
        // to make move cmd

        val makeMoveCmdOut: TestOutputTopic<UUID, String> =
                testDriver.createOutputTopic(Topics.MAKE_MOVE_CMD,
                        uuidSerde.deserializer(), stringSerde.deserializer())

        val expectedMakeMove = MakeMoveCmd(
                gameId = gameId,
                reqId = reqId,
                player = clientPlayerUp,
                coord = clientLastMove.coord
        )

        assertEquals(makeMoveCmdOut.readKeyValue(),
                (KeyValue(sessionId,
                        jsonMapper.writeValueAsString(expectedMakeMove))))

        // artificially introduce a move-made event, so that
        // we can safely state that everyone is on the same page
        val moveMade = MoveMadeEv(gameId = gameId, replyTo = reqId,
            eventId = UUID.randomUUID(), player = clientPlayerUp,
            coord = clientLastMove.coord)

        val moveMadeIn: TestInputTopic<UUID, String> =
                testDriver.createInputTopic(Topics.MOVE_MADE_EV,
                        uuidSerde.serializer(),
                        stringSerde.serializer())

        moveMadeIn.pipeInput(gameId,
                jsonMapper.writeValueAsString(moveMade))

        // check to make sure that sync service outputs
        // a reply that won't require the client to do anything
        val syncReplyOut: TestOutputTopic<UUID, String> =
                testDriver.createOutputTopic(Topics.SYNC_REPLY_EV,
                        uuidSerde.deserializer(), stringSerde.deserializer())

        val expectedSyncReply = SyncReplyEv(
                sessionId = sessionId,
                replyTo = reqId,
                moves = clientMoves,
                gameId = gameId,
                playerUp = clientPlayerUp,
                turn = clientTurn
        )

        assertEquals(syncReplyOut.readKeyValue(),
                (KeyValue(sessionId,
                        jsonMapper.writeValueAsString(expectedSyncReply))))
    }

    @AfterAll
    fun tearDown() {
        testDriver.close()
    }
}

fun setup(): TopologyTestDriver {
    val props = Properties()
    props[StreamsConfig.APPLICATION_ID_CONFIG] = "test-bugout-sync"
    props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = "dummy:1234"
    props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"
    return TopologyTestDriver(Application("dummy-brokers").build(), props)
}
