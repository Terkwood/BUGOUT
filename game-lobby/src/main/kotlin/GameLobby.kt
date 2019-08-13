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

        val allOpenGames: GlobalKTable<String, AllOpenGames> =
            openGamesTable(streamsBuilder)

        val findPublicGameStream: KStream<ReqId, FindPublicGame> =
            streamsBuilder.stream<ReqId, String>(
                Topics.FIND_PUBLIC_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v -> jsonMapper.readValue(v, FindPublicGame::class.java) }

        val fpgKeyJoiner: KeyValueMapper<ReqId, FindPublicGame, String> =
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

        val popPublicGame: KStream<String, GameCommand> =
            publicGameExists.map { _, fpgJoinAllGames ->
                val fpg = fpgJoinAllGames.command

                val someGame =
                    fpgJoinAllGames.store.games.first { g -> g.visibility == Visibility.Public }

                println("Popping public game  ${someGame.gameId.short()}")

                KeyValue(
                    AllOpenGames.TOPIC_KEY,
                    GameCommand(game = someGame, command = Command.Ready)
                )

            }

        popPublicGame.mapValues { v -> jsonMapper.writeValueAsString(v) }.to(
            Topics.OPEN_GAME_COMMANDS,
            Produced.with(Serdes.String(), Serdes.String())
        )

        popPublicGame
            .map { _, v -> KeyValue(v.game.gameId, GameState()) }
            .to(Topics.GAME_STATES_CHANGELOG)

        val changelogNewGame: KStream<GameId, GameStateTurnOnly> =
            streamsBuilder.stream<UUID, String>(
                Topics.GAME_STATES_CHANGELOG,
                Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues { v -> jsonMapper.readValue(v, GameStateTurnOnly::class.java) }

        changelogNewGame.map { k, _ ->
            println("▶          ️${k.short()} READY")
            KeyValue(
                k,
                GameReady(
                    gameId = k,
                    eventId = UUID.randomUUID(),
                    epochMillis = Instant.now().toEpochMilli()
                )
            )
        }.mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(Topics.GAME_READY, Produced.with(Serdes.UUID(), Serdes.String()))


        // TODO
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

        // open a new game
        createGameStream.map { _, v ->
            val newGame = Game(gameId = UUID.randomUUID(), visibility = v.visibility)
            KeyValue(AllOpenGames.TOPIC_KEY, GameCommand(game = newGame, command = Command.Open))
        }.mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(Topics.OPEN_GAME_COMMANDS, Produced.with(Serdes.String(), Serdes.String()))


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

fun openGamesTable(streamsBuilder: StreamsBuilder): GlobalKTable<String, AllOpenGames> {

    /**
     * aggregate data to a local ktable
     * ```sh
     * echo 'ALL:{"games":[]}' | kafkacat -b kafka:9092 -t bugout-game-lobby -K: -P
     * echo 'ALL:{"game": {"gameId":"4c0d9b9a-4040-4f10-8cd0-25a28e332fd7", "visibility":"Public"}, "command": "Open"}' | kafkacat -b kafka:9092 -t bugout-game-lobby-commands -K: -P
     * ```
     */
    val aggregateAll =
        streamsBuilder.stream<String, String>(
            Topics.OPEN_GAME_COMMANDS,
            Consumed.with(Serdes.String(), Serdes.String())
        )
            .groupByKey(
                Serialized.with(Serdes.String(), Serdes.String())
            ).aggregate(
                { AllOpenGames() },
                { _, v, allGames ->
                    allGames.execute(
                        jsonMapper.readValue(v, GameCommand::class.java)
                    )
                    allGames
                },
                Materialized.`as`<String, AllOpenGames, KeyValueStore<Bytes, ByteArray>>(
                    Topics.GAME_LOBBY_STORE_LOCAL
                ).withKeySerde(
                    Serdes.String()
                ).withValueSerde(
                    Serdes.serdeFrom(
                        AllOpenGamesSerializer(),
                        AllOpenGamesDeserializer()
                    )
                )
            )

    aggregateAll.toStream()
        .map { k, v ->
            val json = jsonMapper.writeValueAsString(v)
            println("Aggregated $json")
            KeyValue(k, json)
        }.to(Topics.GAME_LOBBY_CHANGELOG, Produced.with(Serdes.String(), Serdes.String()))

    // expose the aggregated as a global ktable
    // so that we can join against it
    return streamsBuilder.globalTable(
        Topics.GAME_LOBBY_CHANGELOG,
        Materialized.`as`<String, AllOpenGames, KeyValueStore<Bytes, ByteArray>>
            (Topics.GAME_LOBBY_STORE_GLOBAL)
            .withKeySerde(Serdes.String())
            .withValueSerde(
                Serdes.serdeFrom(
                    AllOpenGamesSerializer(),
                    AllOpenGamesDeserializer()
                )
            )
    )
}