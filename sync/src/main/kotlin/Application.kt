
import java.time.temporal.ChronoUnit
import java.util.Properties
import java.util.TimeZone
import java.util.UUID
import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.Topology
import org.apache.kafka.streams.kstream.Consumed
import org.apache.kafka.streams.kstream.JoinWindows
import org.apache.kafka.streams.kstream.Joined
import org.apache.kafka.streams.kstream.KStream
import org.apache.kafka.streams.kstream.KeyValueMapper
import org.apache.kafka.streams.kstream.Produced
import org.apache.kafka.streams.kstream.ValueJoiner
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
        val streamsBuilder = StreamsBuilder()
        val reqSyncStream: KStream<SessionId, ReqSyncCmd> = streamsBuilder
            .stream(
                Topics.REQ_SYNC_CMD,
                Consumed.with(Serdes.UUID(), Serdes.String()))
            .mapValues { v -> jsonMapper.readValue(v,
                            ReqSyncCmd::class.java) }

        reqSyncStream
            .map { k, v ->
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
            .mapValues { v -> jsonMapper.readValue(v, HistoryProvidedEv::class.java) }

        val reqSyncHistProvStream: KStream<SessionId, ReqSyncHistProv> =
            reqSyncStream.join(
                    histProvStream,
                    KeyValueMapper { _: SessionId, // left key
                                     leftValue: ReqSyncCmd ->
                        leftValue.gameId
                    },
                    ValueJoiner { left: ReqSyncCmd, right: HistoryProvidedEv ->
                        ReqSyncHistProv(left, right)
                    },
                    JoinWindows.of(ChronoUnit.MINUTES.duration),
                    Joined.with(
                        Serdes.UUIDSerde(),
                        Serdes.StringSerde(),
                        Serdes.StringSerde()))
                .map { _: UUID, v: ReqSyncHistProv ->
                    KeyValue(v.reqSync.sessionId, v)
                }

        TODO("BRANCHES:")

        TODO("branch 1. client is ahead of server.")
        TODO("branch 1. write to make move cmd")
        TODO("branch 1. in the event that the service layer needs " +
            "to catch up with  the  client's last move, we _must not_ emit " +
            "the  sync reply until   we hear the move confirmed on  " +
            "bugout-move-made-ev")

        TODO("branch 2. no - op: send server view")

        TODO("branch 3 - client is behind server: send server view")
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
