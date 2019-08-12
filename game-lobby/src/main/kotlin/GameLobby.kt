import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import serdes.AllOpenGamesDeserializer
import serdes.AllOpenGamesSerializer
import serdes.jsonMapper
import java.util.*

fun main() {
    TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
    GameLobby("kafka:9092").process()
}

class GameLobby(private val brokers: String) {
    fun process() {
        val streamsBuilder = StreamsBuilder()

        val allOpenGames =
            streamsBuilder.stream<Short, String>(Topics.OPEN_GAMES, Consumed.with(Serdes.Short(), Serdes.String()))
                .groupByKey(
                    Serialized.with(Serdes.Short(), Serdes.String())
                ).aggregate(
                    { AllOpenGames() },
                    { _, v, list ->
                        list.add(
                            jsonMapper.readValue(v, OpenGame::class.java)
                        )
                        list
                    },
                    Materialized.`as`<Short, AllOpenGames, KeyValueStore<Bytes, ByteArray>>(
                        Topics.OPEN_GAMES_STORE_NAME
                    ).withKeySerde(
                        Serdes.Short()
                    ).withValueSerde(Serdes.serdeFrom(AllOpenGamesSerializer(), AllOpenGamesDeserializer()))
                )


        val findPublicGameStream: KStream<ReqId, FindPublicGame> =
            streamsBuilder.stream<ReqId, String>(Topics.FIND_PUBLIC_GAME, Consumed.with(Serdes.UUID(), Serdes.String()))
                .mapValues { v -> jsonMapper.readValue(v, FindPublicGame::class.java) }

        throw NotImplementedError()


        val joinPrivateGameStream: KStream<ReqId, JoinPrivateGame> =
            streamsBuilder.stream<ReqId, String>(
                Topics.JOIN_PRIVATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v -> jsonMapper.readValue(v, JoinPrivateGame::class.java) }

        throw NotImplementedError()


        val createGameStream: KStream<ReqId, CreateGame> =
            streamsBuilder.stream<ReqId, String>(Topics.CREATE_GAME, Consumed.with(Serdes.UUID(), Serdes.String()))
                .mapValues { v -> jsonMapper.readValue(v, CreateGame::class.java) }

        throw NotImplementedError()


        val topology = streamsBuilder.build()

        println(topology.describe())

        val props = Properties()
        props["bootstrap.servers"] = brokers
        props["application.id"] = "bugout-game-lobby"
        props["processing.guarantee"] = "exactly_once"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }
}
