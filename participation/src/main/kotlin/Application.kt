import com.fasterxml.jackson.module.kotlin.jacksonTypeRef
import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.*
import org.apache.kafka.streams.kstream.*
import serdes.KafkaDeserializer
import serdes.KafkaSerializer
import serdes.jsonMapper
import java.time.temporal.ChronoUnit
import java.util.*

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
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "bugout-participation"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        val streams = KafkaStreams(topology, props)

        waitForTopics(Topics.all, props)

        streams.start()
    }

    private fun build(): Topology {
        val streamsBuilder = StreamsBuilder()

        val gameReady: KStream<GameId, GameReady> =
            streamsBuilder.stream<GameId, String>(
                Topics.GAME_READY, Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues {
                    v -> jsonMapper.readValue(v, GameReady::class.java)
            }

        val findPublicGame: KStream<SessionId, FindPublicGame> =
            streamsBuilder.stream<GameId, String>(
                Topics.FIND_PUBLIC_GAME, Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues {
                    v -> jsonMapper.readValue(v, FindPublicGame::class.java)
            }

        val createPrivateGame: KStream<SessionId, CreateGame> =
            streamsBuilder.stream<GameId, String>(
                Topics.CREATE_GAME, Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues {
                    v -> jsonMapper.readValue(v, CreateGame::class.java)
            }.filter { _, v -> v.visibility == Visibility.Private }

        val joinPrivateGame: KStream<SessionId, JoinPrivateGame> =
            streamsBuilder.stream<GameId, String>(
                Topics.JOIN_PRIVATE_GAME, Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues {
                    v -> jsonMapper.readValue(v, JoinPrivateGame::class.java)
            }

        listOf(gameReady, findPublicGame, joinPrivateGame, createPrivateGame)
            .forEach { stream ->  stream.foreach {
                    _,v ->
                run {
                    println("$v")
                    v
                }
            }}

        val gameReadyBySessionId: KStream<SessionId, GameReady> = gameReady
            .map { _, v -> KeyValue(v.sessions.first, v)}
            .merge(gameReady.map { _, v -> KeyValue(v.sessions.second, v)})

        gameReadyBySessionId
            .foreach { sessionId, gr ->
                println("GR sessionId $sessionId -> ${gr.gameId}")
            }

        val cpgGr: KStream<GameId, Pair<CreateGame, GameReady>> =
            createPrivateGame.join(
                gameReadyBySessionId,
                { left: CreateGame, right: GameReady -> Pair(left, right) },
                JoinWindows.of(ChronoUnit.HOURS.duration),
                Joined.with(Serdes.UUID(),
                    Serdes.serdeFrom(KafkaSerializer(), KafkaDeserializer(jacksonTypeRef())),
                    Serdes.serdeFrom(KafkaSerializer(), KafkaDeserializer(jacksonTypeRef()))))
                .map { _, v -> KeyValue(v.second.gameId, v) }

        // Carefully avoiding overlap by joining against gameReady, not gameReadyBySessionId
        val jpgGr: KStream<GameId, Pair<JoinPrivateGame, GameReady>> =
            joinPrivateGame.map{_, v -> KeyValue(v.gameId,v)}.join(
                gameReady,
                {left: JoinPrivateGame, right:GameReady -> Pair(left, right) },
                JoinWindows.of(ChronoUnit.HOURS.duration),
                Joined.with(Serdes.UUID(),
                    Serdes.serdeFrom(KafkaSerializer(), KafkaDeserializer(jacksonTypeRef())),
                    Serdes.serdeFrom(KafkaSerializer(), KafkaDeserializer(jacksonTypeRef()))
                    ))
                .map {_, v -> KeyValue(v.second.gameId, v) }

        val privateReady: KStream<GameId, Triple<CreateGame, JoinPrivateGame, GameReady>> = cpgGr.join(jpgGr,
            { left, right -> Triple(left.first, right.first, left.second)},
            JoinWindows.of(ChronoUnit.HOURS.duration),
            Joined.with(Serdes.UUID(),
                Serdes.serdeFrom(KafkaSerializer(), KafkaDeserializer(jacksonTypeRef())),
                Serdes.serdeFrom(KafkaSerializer(), KafkaDeserializer(jacksonTypeRef()))
            ))

        privateReady.foreach { _ , it -> println("join private + create private + game ready $it") }

        return streamsBuilder.build()
    }

    private fun waitForTopics(topics: Array<String>, props:
    Properties) {
        print("⏲ Waiting for topics ")
        val client = AdminClient.create(props)

        var topicsReady = false
        while(!topicsReady) {
            val found = client.listTopics().names().get()

            val diff = topics.subtract(found.filterNotNull())

            topicsReady = diff.isEmpty()

            if (!topicsReady) Thread.sleep(333)
            print(".")
        }

        println(" done! 🏁")
    }
}