import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.StringSerializer
import org.apache.kafka.common.serialization.UUIDSerializer
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.*
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
        val topology = build()

        println(topology.describe())

        val props = Properties()
        props[StreamsConfig.BOOTSTRAP_SERVERS_CONFIG] = brokers
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "bugout-game-lobby"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }


    fun build(): Topology {
        val streamsBuilder = StreamsBuilder()

        val allOpenGames: GlobalKTable<String, AllOpenGames> =
            buildGameLobbyTable(streamsBuilder)

        buildPublicGameStreams(streamsBuilder, allOpenGames)

        val changelogNewGame: KStream<GameId, GameStateTurnOnly> =
            streamsBuilder.stream<UUID, String>(
                Topics.GAME_STATES_CHANGELOG,
                Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues { v ->
                jsonMapper.readValue(
                    v,
                    GameStateTurnOnly::class.java
                )
            }

        val changelogTurnOne =
            changelogNewGame
                .filter { _, gameState -> gameState.turn == 1 }

        // we need to join game states changelog against the lobby
        // so that we can figure out the creator of the game,
        // and correctly announce game ready
        // game states changelog is a stream, here
        val gslKVMapper: KeyValueMapper<GameId, GameStateTurnOnly,
                String> =
            KeyValueMapper { _: GameId, // left key
                             _: GameStateTurnOnly ->
                // left value

                // use a trivial join, so that all queries are routed to the same store
                AllOpenGames.TRIVIAL_KEY
            }

        val gslValueJoiner: ValueJoiner<GameStateTurnOnly,
                AllOpenGames, GameStateLobby> =
            ValueJoiner { leftValue:
                          GameStateTurnOnly,
                          rightValue:
                          AllOpenGames ->
                GameStateLobby(leftValue, rightValue)
            }

        val gameStateLobby =
            changelogTurnOne.join(
                allOpenGames,
                gslKVMapper, gslValueJoiner
            )

        val waitForOpponent = gameStateLobby.filter { k, v ->
            v.lobby.games.any { it.gameId == k }
        }.map { k, v ->
            println("▶          ️${k.short()} READY")
            val creator = v.lobby.games.find { it.gameId == k }?.creator!!
            KeyValue(
                k,
                WaitForOpponent(
                    gameId = k,
                    eventId = UUID.randomUUID(),
                    clientId = creator
                )
            )
        }

        /* final
        .mapValues { v -> jsonMapper.writeValueAsString(v) }
        .to(
            Topics.GAME_READY,
            Produced.with(Serdes.UUID(), Serdes.String())
        )

         */


        // TODO
        val joinPrivateGameStream: KStream<ClientId, JoinPrivateGame> =
            streamsBuilder.stream<ClientId, String>(
                Topics.JOIN_PRIVATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    jsonMapper.readValue(
                        v,
                        JoinPrivateGame::class.java
                    )
                }

        // TODO throw NotImplementedError()


        val createGameStream: KStream<ClientId, CreateGame> =
            streamsBuilder.stream<ClientId, String>(
                Topics.CREATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    jsonMapper.readValue(
                        v,
                        CreateGame::class.java
                    )
                }

        // open a new game
        createGameStream.map { _, v ->
            val newGame =
                Game(
                    gameId = v.gameId,
                    visibility = v.visibility,
                    creator = v.clientId
                )
            KeyValue(
                AllOpenGames.TRIVIAL_KEY,
                GameCommand(game = newGame, command = Command.Open)
            )
        }.mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.GAME_LOBBY_COMMANDS,
                Produced.with(Serdes.String(), Serdes.String())
            )



        return streamsBuilder.build()
    }

    /**
     * aggregate data to a local ktable
     * ```sh
     * echo 'ALL:{"games":[]}' | kafkacat -b kafka:9092 -t bugout-game-lobby -K: -P
     * echo 'ALL:{"game": {"gameId":"4c0d9b9a-4040-4f10-8cd0-25a28e332fd7", "visibility":"Public"}, "command": "Open"}' | kafkacat -b kafka:9092 -t bugout-game-lobby-commands -K: -P
     * ```
     */
    private fun buildGameLobbyTable(streamsBuilder: StreamsBuilder): GlobalKTable<String, AllOpenGames> {

        val aggregateAll =
            streamsBuilder.stream<String, String>(
                Topics.GAME_LOBBY_COMMANDS,
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
            }.to(
                Topics.GAME_LOBBY_CHANGELOG,
                Produced.with(Serdes.String(), Serdes.String())
            )

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

    private fun buildPublicGameStreams(
        streamsBuilder: StreamsBuilder,
        allOpenGames: GlobalKTable<String, AllOpenGames>
    ) {
        val findPublicGameStream: KStream<ClientId, FindPublicGame> =
            streamsBuilder.stream<ClientId, String>(
                Topics.FIND_PUBLIC_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    jsonMapper.readValue(
                        v,
                        FindPublicGame::class.java
                    )
                }

        val fpgKeyJoiner: KeyValueMapper<ClientId, FindPublicGame, String> =
            KeyValueMapper { _: ClientId, // left key
                             _: FindPublicGame ->
                // left value

                // use a trivial join, so that all queries are routed to the same store
                AllOpenGames.TRIVIAL_KEY
            }

        val fpgValueJoiner: ValueJoiner<FindPublicGame, AllOpenGames, FindPublicGameAllOpenGames> =
            ValueJoiner { leftValue:
                          FindPublicGame,
                          rightValue:
                          AllOpenGames ->
                FindPublicGameAllOpenGames(leftValue, rightValue)
            }


        val fpgJoinAllOpenGames =
            findPublicGameStream.join(
                allOpenGames,
                fpgKeyJoiner,
                fpgValueJoiner
            )

        val fpgBranches =
            fpgJoinAllOpenGames
                .kbranch(
                    { _, fo ->
                        fo.store.games.any { g -> g.visibility == Visibility.Public }
                    },
                    { _, fo ->
                        !fo.store.games.any { g -> g.visibility == Visibility.Public }
                    }
                )

        val publicGameExists: KStream<ClientId, FindPublicGameAllOpenGames> =
            fpgBranches[0]

        val noPublicGameExists: KStream<ClientId, FindPublicGameAllOpenGames> =
            fpgBranches[1]


        // if someone was looking for a public game and
        // didn't find one
        noPublicGameExists
            .map { _, fo ->
                KeyValue(
                    fo.command.clientId,
                    // game ID randomly generated here
                    CreateGame(
                        fo.command.clientId,
                        Visibility.Public
                    )
                )
            }.mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.CREATE_GAME,
                Produced.with(Serdes.UUID(), Serdes.String())
            )


        /**
         * The ClientId key is the person finding a game
         * The creator of the game is buried in the game lobby (someGame)
         */
        val popPublicGame: KStream<ClientId, GameCommand> =
            publicGameExists.map { _, fpgJoinAllGames ->
                val fpg = fpgJoinAllGames.command

                val someGame =
                    fpgJoinAllGames.store.games.first { g -> g.visibility == Visibility.Public }

                println("Popping public game  ${someGame.gameId.short()}")

                KeyValue(
                    fpgJoinAllGames.command.clientId,
                    GameCommand(game = someGame, command = Command.Ready)
                )

            }

        popPublicGame.map { finderClientId, gameCommand ->
            KeyValue(
                gameCommand.game.gameId,
                GameReady(
                    gameCommand.game.gameId,
                    Pair(gameCommand.game.creator, finderClientId)
                )
            )
        }.mapValues { it -> jsonMapper.writeValueAsString(it) }
            .to(
                Topics
                    .GAME_READY,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        popPublicGame.map { _, v ->
            KeyValue(
                AllOpenGames.TRIVIAL_KEY,
                jsonMapper.writeValueAsString(v)
            )
        }.to(
            Topics.GAME_LOBBY_COMMANDS,
            Produced.with(Serdes.String(), Serdes.String())
        )

        popPublicGame
            .map { _, v -> KeyValue(v.game.gameId, GameState()) }
            .to(Topics.GAME_STATES_CHANGELOG)

    }
}


