import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.KeyValue
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

        // aggregate data as it comes in
        // this is done with a local ktable
        val aggregateAll =
            streamsBuilder.stream<Short, String>(
                Topics.OPEN_GAME_COMMANDS,
                Consumed.with(Serdes.Short(), Serdes.String())
            )
                .groupByKey(
                    Serialized.with(Serdes.Short(), Serdes.String())
                ).aggregate(
                    { AllOpenGames() },
                    { _, v, allGames ->
                        allGames.update(
                            jsonMapper.readValue(v, OpenGameCommand::class.java)
                        )
                        allGames
                    },
                    Materialized.`as`<Short, AllOpenGames, KeyValueStore<Bytes, ByteArray>>(
                        Topics.OPEN_GAMES_STORE_NAME_LOCAL
                    ).withKeySerde(
                        Serdes.Short()
                    ).withValueSerde(Serdes.serdeFrom(AllOpenGamesSerializer(), AllOpenGamesDeserializer()))
                )

        aggregateAll.toStream().map { k, v -> KeyValue(k, jsonMapper.writeValueAsString(v)) }
            .to(Topics.OPEN_GAMES, Produced.with(Serdes.Short(), Serdes.String()))

        // expose the aggregated as a global ktable
        // so that we can join against it
        val allOpenGames: GlobalKTable<Short, AllOpenGames> =
            streamsBuilder.globalTable(
                Topics.OPEN_GAMES,
                Materialized.`as`<Short, AllOpenGames, KeyValueStore<Bytes, ByteArray>>
                    (Topics.OPEN_GAMES_STORE_NAME_GLOBAL)
                    .withKeySerde(Serdes.Short())
                    .withValueSerde(Serdes.serdeFrom(AllOpenGamesSerializer(), AllOpenGamesDeserializer()))
            )

        val findPublicGameStream: KStream<ReqId, FindPublicGame> =
            streamsBuilder.stream<ReqId, String>(Topics.FIND_PUBLIC_GAME, Consumed.with(Serdes.UUID(), Serdes.String()))
                .mapValues { v -> jsonMapper.readValue(v, FindPublicGame::class.java) }

        val fpgKeyJoiner: KeyValueMapper<ReqId, FindPublicGame, Short> =
            KeyValueMapper { _: ReqId, // left key
                             _: FindPublicGame ->
                // left value

                // use a trivial join, so that all queries are routed to the same store
                AllOpenGames.TOPIC_KEY
            }

        val fpgValueJoiner: ValueJoiner<FindPublicGame, AllOpenGames, FindPublicGameAllOpenGames> =
            ValueJoiner { leftValue:
                          FindPublicGame,
                          rightValue:
                          AllOpenGames ->
                FindPublicGameAllOpenGames(leftValue, rightValue)
            }

        val fpgJoinAllOpenGames =
            findPublicGameStream.leftJoin(allOpenGames, fpgKeyJoiner, fpgValueJoiner)

        val fpgBranches =
            fpgJoinAllOpenGames
                .kbranch({ _, fpgOpenGames ->
                    fpgOpenGames.store.games
                        .any { g -> g.visibility == Visibility.Public }
                })

        when(fpgBranches.size) {
            0 -> print("nothing")
            1 -> print("one")
            2 -> print("two")
        }
        

        // TODO throw NotImplementedError()


        val joinPrivateGameStream: KStream<ReqId, JoinPrivateGame> =
            streamsBuilder.stream<ReqId, String>(
                Topics.JOIN_PRIVATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v -> jsonMapper.readValue(v, JoinPrivateGame::class.java) }

        // TODO throw NotImplementedError()


        val createGameStream: KStream<ReqId, CreateGame> =
            streamsBuilder.stream<ReqId, String>(Topics.CREATE_GAME, Consumed.with(Serdes.UUID(), Serdes.String()))
                .mapValues { v -> jsonMapper.readValue(v, CreateGame::class.java) }

        // TODO throw NotImplementedError()


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
