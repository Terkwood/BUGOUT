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
import java.time.Instant
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
                        allGames.execute(
                            jsonMapper.readValue(v, GameCommand::class.java)
                        )
                        allGames
                    },
                    Materialized.`as`<Short, AllOpenGames, KeyValueStore<Bytes, ByteArray>>(
                        Topics.OPEN_GAMES_STORE_NAME_LOCAL
                    ).withKeySerde(
                        Serdes.Short()
                    ).withValueSerde(
                        Serdes.serdeFrom(
                            AllOpenGamesSerializer(),
                            AllOpenGamesDeserializer()
                        )
                    )
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
                    .withValueSerde(
                        Serdes.serdeFrom(
                            AllOpenGamesSerializer(),
                            AllOpenGamesDeserializer()
                        )
                    )
            )

        val findPublicGameStream: KStream<ReqId, FindPublicGame> =
            streamsBuilder.stream<ReqId, String>(
                Topics.FIND_PUBLIC_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
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

        val publicGameExists = fpgBranches[0]

        val popPublicGame: KStream<Short, GameCommand> =
            publicGameExists.map { _, fpgJoinAllGames ->
                val fpg = fpgJoinAllGames.command

                val someGame =
                    fpgJoinAllGames.store.games.first { g -> g.visibility == Visibility.Public }

                KeyValue(
                    AllOpenGames.TOPIC_KEY,
                    GameCommand(game = someGame, command = Command.Ready)
                )

            }

        popPublicGame.to(Topics.OPEN_GAME_COMMANDS)

        popPublicGame
            .map { _, v -> KeyValue(v.game.gameId, GameState()) }
            .to(Topics.GAME_STATES_CHANGELOG_TOPIC)
        
        val changelogNewGame: KStream<GameId, GameStateTurnOnly> =
            streamsBuilder.stream<UUID, String>(
                Topics.GAME_STATES_CHANGELOG_TOPIC,
                Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues { v -> jsonMapper.readValue(v, GameStateTurnOnly::class.java) }

        changelogNewGame.map { k, _ ->
            KeyValue(
                k,
                GameReady(
                    gameId = k,
                    eventId = UUID.randomUUID(),
                    epochMillis = Instant.now().toEpochMilli()
                )
            )
        }.to(Topics.GAME_READY)

        val joinPrivateGameStream: KStream<ReqId, JoinPrivateGame> =
            streamsBuilder.stream<ReqId, String>(
                Topics.JOIN_PRIVATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v -> jsonMapper.readValue(v, JoinPrivateGame::class.java) }

        // TODO throw NotImplementedError()


        val createGameStream: KStream<ReqId, CreateGame> =
            streamsBuilder.stream<ReqId, String>(
                Topics.CREATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
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
