
import com.fasterxml.jackson.module.kotlin.jacksonTypeRef
import java.time.temporal.ChronoUnit
import java.util.Properties
import java.util.TimeZone
import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.Topology
import org.apache.kafka.streams.kstream.Consumed
import org.apache.kafka.streams.kstream.JoinWindows
import org.apache.kafka.streams.kstream.KStream
import org.apache.kafka.streams.kstream.Produced
import org.apache.kafka.streams.kstream.StreamJoined
import serdes.KafkaDeserializer
import serdes.KafkaSerializer
import serdes.jsonMapper

const val BROKERS = "kafka:9092"

fun main() {
    TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
    Application(BROKERS).process()
}

class Application(private val brokers: String) {
    fun process() {
        val topology = build()

        println(topology.describe())

        val props = Properties()
        props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = brokers
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "bugout-sync"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        val streams = KafkaStreams(topology, props)

        waitForTopics(Topics.all, props)

        streams.start()
    }

    fun build(): Topology {
        val joinDur = JoinWindows.of(ChronoUnit.MINUTES.duration)
        val streamsBuilder = StreamsBuilder()
        val reqSyncStream: KStream<SessionId, ReqSyncCmd> = streamsBuilder
            .stream(
                Topics.REQ_SYNC_CMD,
                Consumed.with(Serdes.UUID(), Serdes.String()))
            .mapValues { v -> jsonMapper.readValue(v,
                            ReqSyncCmd::class.java) }

        reqSyncStream
            .map { _, v ->
                KeyValue(v.gameId,
                    ProvideHistoryCmd(gameId = v.gameId, reqId = v.reqId))
            }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(Topics.PROVIDE_HISTORY_CMD,
                Produced.with(Serdes.UUID(), Serdes.String()))

        val histProvStream: KStream<GameId, HistoryProvidedEv> = streamsBuilder
            .stream(
                Topics.HISTORY_PROVIDED_EV,
                Consumed.with(Serdes.UUID(), Serdes.String()))
            .mapValues { v ->
                jsonMapper.readValue(v, HistoryProvidedEv::class.java)
            }

        val reqSyncByGameId = reqSyncStream
            .map { _, v -> KeyValue(v.gameId, v) }

        val histJoined: KStream<GameId, HistProvReply> = reqSyncByGameId.join(
                histProvStream,
                { left: ReqSyncCmd,
                  right: HistoryProvidedEv ->
                    val maybeLastMove = right.moves.lastOrNull()
                    val systemPlayerUp: Player =
                            if (maybeLastMove == null) Player.BLACK else
                                otherPlayer(maybeLastMove.player)
                    val systemTurn: Int =
                            (maybeLastMove?.turn ?: 0) + 1

                    HistProvReply(left,
                            right,
                            systemTurn, systemPlayerUp
                    )
                },
                joinDur,
                StreamJoined.with(
                        Serdes.UUID(),
                        Serdes.serdeFrom(
                                KafkaSerializer(),
                                KafkaDeserializer(jacksonTypeRef())),
                        Serdes.serdeFrom(
                                KafkaSerializer(),
                                KafkaDeserializer(jacksonTypeRef()))))

        val histReplyStream: KStream<SessionId, HistProvReply> = histJoined
                .filter { _, v -> v.reqSync.reqId == v.histProv.replyTo }
                .map { _, v: HistProvReply ->
                    KeyValue(v.reqSync.sessionId, v)
                }

        val branches = histReplyStream.kbranch(
            // client is ahead of server by a single turn
            // and their move needs to be processed
            { _: SessionId, hpr: HistProvReply ->
                isClientAheadByOneTurn(hpr)
            },
            // in every other case, we should send the server's view:
            // - no op: client is caught up
            // - client is behind by one move
            // - client has a state which we cannot reconcile
            //            ...(but maybe they can fix themselves)
            { _: SessionId, hpr: HistProvReply ->
                !isClientAheadByOneTurn(hpr) }
        )

        val clientAheadByOneTurnBranch = branches[0]
        clientAheadByOneTurnBranch.map { _, v ->
                val missedMove = v.reqSync.lastMove!! // checked null above
                KeyValue(v.reqSync.gameId,
                    MakeMoveCmd(gameId = v
                        .reqSync.gameId, reqId = v.reqSync.reqId,
                        player = missedMove.player,
                        coord = missedMove.coord)) }
            .mapValues { v ->
                jsonMapper.writeValueAsString(v)
            }
            .to(Topics.MAKE_MOVE_CMD,
                Produced.with(Serdes.UUID(), Serdes.String()))

        val moveMadeStream: KStream<GameId, MoveMadeEv> = streamsBuilder
            .stream(
                Topics.MOVE_MADE_EV,
                Consumed.with(Serdes.UUID(), Serdes.String()))
            .mapValues { v ->
                jsonMapper.readValue(v, MoveMadeEv::class.java)
            }

        val clientAheadByReqId = clientAheadByOneTurnBranch
            .map { _, v ->
                KeyValue(v.reqSync.reqId, v)
            }

        val moveMadeByReqId =
            moveMadeStream.map { _, v ->
                KeyValue(v.replyTo, v)
            }

        val histProvMoveMadeReplies: KStream<ReqId, SystemMoved> =
            clientAheadByReqId
                    .join(
                        moveMadeByReqId,
                        { l, r -> SystemMoved(l, r) },
                        joinDur,
                            StreamJoined.with(
                                    Serdes.UUID(),
                                    Serdes.serdeFrom(
                                            KafkaSerializer(),
                                            KafkaDeserializer(jacksonTypeRef())),
                                    Serdes.serdeFrom(
                                            KafkaSerializer(),
                                            KafkaDeserializer(jacksonTypeRef()))))

        val clientMoveComputed: KStream<SessionId, SyncReplyEv> =
            histProvMoveMadeReplies.map { reqId, v ->
                val allMoves = ArrayList<Move>()
                allMoves.addAll(v.hist.histProv.moves)
                val theTurn = (
                        v.hist.histProv.moves.lastOrNull()?.turn ?: 0
                        ) + 1
                allMoves.add(Move(
                    v.moved.player,
                    v.moved.coord,
                    theTurn))

                KeyValue(
                    v.hist.reqSync.sessionId,
                    SyncReplyEv(
                        sessionId = v.hist.reqSync.sessionId,
                        gameId = v.hist.reqSync.gameId,
                        replyTo = reqId,
                        moves = allMoves,
                        turn = theTurn + 1,
                        playerUp = otherPlayer(v.moved.player)
                    )
                )
            }

        val sendServerViewBranch = branches[1]

        val serverViewSyncReply = sendServerViewBranch
            .mapValues { histProvReply ->
                SyncReplyEv(
                    sessionId = histProvReply.reqSync.sessionId,
                    gameId = histProvReply.reqSync.gameId,
                    replyTo = histProvReply.reqSync.reqId,
                    moves = histProvReply.histProv.moves,
                    playerUp = histProvReply.systemPlayerUp,
                    turn = histProvReply.systemTurn)
            }

        serverViewSyncReply.merge(clientMoveComputed)
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(Topics.SYNC_REPLY_EV,
                Produced.with(Serdes.UUID(), Serdes.String()))

        return streamsBuilder.build()
    }

    private fun waitForTopics(
        topics: Array<String>,
        props:
            Properties
    ) {
        print("⏲ Waiting for topics ")
        val client = AdminClient.create(props)

        var topicsReady = false
        while (!topicsReady) {
            val found = client.listTopics().names().get()

            val diff = topics.subtract(found.filterNotNull())

            topicsReady = diff.isEmpty()

            if (!topicsReady) Thread.sleep(333)
            print(".")
        }

        println(" done! 🏁")
    }
}

private fun isClientAheadByOneTurn(hpr: HistProvReply): Boolean =
    hpr.reqSync.turn == hpr.systemTurn + 1 &&
            hpr.reqSync.playerUp == otherPlayer(hpr.systemPlayerUp) &&
            hpr.reqSync.lastMove != null
