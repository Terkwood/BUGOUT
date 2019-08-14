import org.apache.kafka.common.serialization.Serdes
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
    Application("kafka:9092").process()
}

class Application(private val brokers: String) {
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

        val gameLobby: GlobalKTable<String, GameLobby> =
            buildGameLobbyTable(streamsBuilder)

        buildPublicGameStreams(streamsBuilder, gameLobby)

        val gameStatesChangeLog: KStream<GameId, GameStateTurnOnly> =
            streamsBuilder.stream<UUID, String>(
                Topics.GAME_STATES_CHANGELOG,
                Consumed.with(Serdes.UUID(), Serdes.String())
            ).mapValues { v ->
                jsonMapper.readValue(
                    v,
                    GameStateTurnOnly::class.java
                )
            }

        val gameStatesTurnOne =
            gameStatesChangeLog
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
                GameLobby.TRIVIAL_KEY
            }

        val gslValueJoiner: ValueJoiner<GameStateTurnOnly,
                GameLobby, GameStateLobby> =
            ValueJoiner { leftValue:
                          GameStateTurnOnly,
                          rightValue:
                          GameLobby ->
                GameStateLobby(leftValue, rightValue)
            }

        val turnOneGameStateLobby =
            gameStatesTurnOne.join(
                gameLobby,
                gslKVMapper, gslValueJoiner
            )

        val waitForOpponent =
            turnOneGameStateLobby.filter { k, v ->
                v.lobby.games.any { it.gameId == k }
            }.map { k, v ->
                println("!          ️${k.short()} WAIT")
                // based on changelogTurnOne, we know that we can bypass this
                // null check
                val creator = v.lobby.games.find { it.gameId == k }?.creator!!
                KeyValue(
                    creator,
                    WaitForOpponent(
                        gameId = k,
                        eventId = UUID.randomUUID(),
                        clientId = creator
                    )
                )
            }
        waitForOpponent.mapValues { v -> jsonMapper.writeValueAsString(v) }.to(
            Topics
                .WAIT_FOR_OPPONENT, Produced.with(
                Serdes.UUID(), Serdes
                    .String()
            )
        )


        val joinPrivateGameStream: KStream<ClientId, JoinPrivateGame> =
            streamsBuilder.stream<ClientId, String>(
                Topics.JOIN_PRIVATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    println("READ A JOIN PRIV REQUEST   ... ")
                    jsonMapper.readValue(
                        v,
                        JoinPrivateGame::class.java
                    )
                }

        val joinPrivateKVM: KeyValueMapper<ClientId, JoinPrivateGame,
                String> =
            KeyValueMapper { _: ClientId, // left key
                             _: JoinPrivateGame ->
                // left value

                // use a trivial join, so that all queries are routed to the same store
                GameLobby.TRIVIAL_KEY
            }

        val joinPrivateVJ: ValueJoiner<JoinPrivateGame, GameLobby,
                JoinPrivateGameLobby> =
            ValueJoiner { leftValue:
                          JoinPrivateGame,
                          rightValue:
                          GameLobby ->
                JoinPrivateGameLobby(leftValue, rightValue)
            }

        val joinPrivateLobby =
            joinPrivateGameStream.join(
                gameLobby,
                joinPrivateKVM,
                joinPrivateVJ
            )

        val joinPrivateLobbyBranches =
            joinPrivateLobby
                .kbranch(
                    { _, jl ->
                        jl.lobby.games.any { g ->
                            g.visibility == Visibility.Private
                                    && g.gameId == jl.command.gameId
                        }
                    },
                    { _, jl ->
                        !jl.lobby.games.any { g ->
                            g.visibility == Visibility.Private
                                    && g.gameId == jl.command.gameId
                        }
                    }
                )


        val joinPrivateSuccess: KStream<ClientId, JoinPrivateGameLobby> =
            joinPrivateLobbyBranches[0]

        val joinPrivateFailure: KStream<ClientId, JoinPrivateGameLobby> =
            joinPrivateLobbyBranches[1]


        // reject invalid game IDs
        joinPrivateFailure
            .map { _, jl ->
                println("HEY THERE GUY ➿")
                KeyValue(
                    jl.command.clientId,
                    // game ID randomly generated here
                    PrivateGameRejected(
                        clientId = jl.command.clientId,
                        gameId = jl.command.gameId
                    )
                )
            }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.PRIVATE_GAME_REJECTED,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        // TODO priv join success


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
                GameLobby.TRIVIAL_KEY,
                GameLobbyCommand(
                    game = newGame,
                    lobbyCommand = LobbyCommand.Open
                )
            )
        }.mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.GAME_LOBBY_COMMANDS,
                Produced.with(Serdes.String(), Serdes.String())
            )

        // write an empty game state to the game states changelog
        createGameStream.map { _, cg ->
            KeyValue(cg.gameId, GameState())
        }.mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.GAME_STATES_CHANGELOG, Produced.with(
                    Serdes.UUID(),
                    Serdes.String()
                )
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
    private fun buildGameLobbyTable(streamsBuilder: StreamsBuilder): GlobalKTable<String, GameLobby> {

        val aggregateAll =
            streamsBuilder.stream<String, String>(
                Topics.GAME_LOBBY_COMMANDS,
                Consumed.with(Serdes.String(), Serdes.String())
            )
                .groupByKey(
                    Serialized.with(Serdes.String(), Serdes.String())
                ).aggregate(
                    { GameLobby() },
                    { _, v, allGames ->
                        allGames.execute(
                            jsonMapper.readValue(
                                v,
                                GameLobbyCommand::class.java
                            )
                        )
                        allGames
                    },
                    Materialized.`as`<String, GameLobby, KeyValueStore<Bytes, ByteArray>>(
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
            Materialized.`as`<String, GameLobby, KeyValueStore<Bytes, ByteArray>>
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
        gameLobby: GlobalKTable<String, GameLobby>
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
                GameLobby.TRIVIAL_KEY
            }

        val fpgValueJoiner: ValueJoiner<FindPublicGame, GameLobby, FindPublicGameLobby> =
            ValueJoiner { leftValue:
                          FindPublicGame,
                          rightValue:
                          GameLobby ->
                FindPublicGameLobby(leftValue, rightValue)
            }


        val fpgLobby =
            findPublicGameStream.join(
                gameLobby,
                fpgKeyJoiner,
                fpgValueJoiner
            )

        val fpgLobbyBranches =
            fpgLobby
                .kbranch(
                    { _, fl ->
                        fl.lobby.games.any { g -> g.visibility == Visibility.Public }
                    },
                    { _, fl ->
                        !fl.lobby.games.any { g -> g.visibility == Visibility.Public }
                    }
                )

        val publicGameExists: KStream<ClientId, FindPublicGameLobby> =
            fpgLobbyBranches[0]

        val noPublicGameExists: KStream<ClientId, FindPublicGameLobby> =
            fpgLobbyBranches[1]


        // if someone was looking for a public game and
        // didn't find one
        noPublicGameExists
            .map { _, fo ->
                KeyValue(
                    fo.command.clientId,
                    // game ID randomly generated here
                    CreateGame(
                        clientId = fo.command.clientId,
                        visibility = Visibility.Public,
                        gameId = UUID.randomUUID()
                    )
                )
            }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.CREATE_GAME,
                Produced.with(Serdes.UUID(), Serdes.String())
            )


        /**
         * The ClientId key is the person finding a game
         * The creator of the game is buried in the game lobby (someGame)
         */
        val popPublicGame: KStream<ClientId, GameLobbyCommand> =
            publicGameExists.map { _, fpgJoinAllGames ->
                val fpg = fpgJoinAllGames.command

                val someGame =
                    fpgJoinAllGames.lobby.games.first { g -> g.visibility == Visibility.Public }

                println("Popping public game  ${someGame.gameId.short()}")

                KeyValue(
                    fpgJoinAllGames.command.clientId,
                    GameLobbyCommand(
                        game = someGame,
                        lobbyCommand = LobbyCommand.Ready
                    )
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
                GameLobby.TRIVIAL_KEY,
                jsonMapper.writeValueAsString(v)
            )
        }.to(
            Topics.GAME_LOBBY_COMMANDS,
            Produced.with(Serdes.String(), Serdes.String())
        )

        popPublicGame
            .map { _, v -> KeyValue(v.game.gameId, GameState()) }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.GAME_STATES_CHANGELOG,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

    }
}


