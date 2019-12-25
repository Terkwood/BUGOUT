import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.*
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import serdes.AllOpenGamesDeserializer
import serdes.AllOpenGamesSerializer
import serdes.jsonMapper
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
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "bugout-game-lobby"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        val streams = KafkaStreams(topology, props)

        waitForTopics(Topics.all, props)

        streams.start()
    }

    fun build(): Topology {
        val streamsBuilder = StreamsBuilder()

        val gameLobby: GlobalKTable<String, GameLobby> =
            buildGameLobbyTable(streamsBuilder)

        buildPublicGameStreams(streamsBuilder, gameLobby)

        buildAbandonGameStreams(streamsBuilder, gameLobby)

        val joinPrivateGameStream: KStream<SessionId, JoinPrivateGame> =
            streamsBuilder.stream<SessionId, String>(
                Topics.JOIN_PRIVATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    jsonMapper.readValue(
                        v,
                        JoinPrivateGame::class.java
                    )
                }

        val joinPrivateKVM: KeyValueMapper<SessionId, JoinPrivateGame,
                String> =
            KeyValueMapper { _: SessionId,           // left key
                             _: JoinPrivateGame ->  // left value

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


        val joinPrivateSuccess: KStream<SessionId, JoinPrivateGameLobby> =
            joinPrivateLobbyBranches[0]

        val joinPrivateFailure: KStream<SessionId, JoinPrivateGameLobby> =
            joinPrivateLobbyBranches[1]

        /**
         * The SessionId key is the person finding a game
         * The creator of the game is buried in the game lobby (someGame)
         */
        val popPrivateGame: KStream<SessionId, GameLobbyCommand> =
            joinPrivateSuccess.map { sid, jpgLobby ->
                val jpg = jpgLobby.command
                val lobby = jpgLobby.lobby

                val someGame =
                    lobby.games.first { g ->
                        g.visibility == Visibility.Private &&
                                g.gameId == jpg.gameId
                    }

                KeyValue(
                    sid,
                    GameLobbyCommand(
                        game = someGame,
                        lobbyCommand = LobbyCommand.Ready
                    )
                )

            }


        popPrivateGame.map { finderSessionId, gameCommand ->
            KeyValue(
                gameCommand.game.gameId,
                GameReady(
                    gameCommand.game.gameId,
                    Pair(gameCommand.game.creator, finderSessionId),
                    boardSize = gameCommand.game.boardSize
                )
            )
        }.mapValues { it -> jsonMapper.writeValueAsString(it) }
            .to(
                Topics
                    .GAME_READY,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        popPrivateGame.map { _, v ->
            KeyValue(
                GameLobby.TRIVIAL_KEY,
                jsonMapper.writeValueAsString(v)
            )
        }.to(
            Topics.GAME_LOBBY_COMMANDS,
            Produced.with(Serdes.String(), Serdes.String())
        )

        popPrivateGame
            .map { _, v ->
                KeyValue(v.game.gameId,
                    GameState(board = Board(size = v.game.boardSize))) }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.GAME_STATES_CHANGELOG,
                Produced.with(Serdes.UUID(), Serdes.String())
            )


        // reject invalid game IDs
        joinPrivateFailure
            .map { sid, jl ->
                KeyValue(
                    sid,
                    // game ID randomly generated here
                    PrivateGameRejected(
                        clientId = jl.command.clientId,
                        gameId = jl.command.gameId,
                        sessionId = jl.command.sessionId
                    )
                )
            }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.PRIVATE_GAME_REJECTED,
                Produced.with(Serdes.UUID(), Serdes.String())
            )


        val createGameStream: KStream<SessionId, CreateGame> =
            streamsBuilder.stream<SessionId, String>(
                Topics.CREATE_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    jsonMapper.readValue(
                        v,
                        CreateGame::class.java
                    )
                }

        // send a wait for opponent event when game is created
        createGameStream.map { creator, v ->
            KeyValue(
                creator,
                WaitForOpponent(
                    gameId = v.gameId,
                    eventId = UUID.randomUUID(),
                    sessionId = creator,
                    visibility = v.visibility
                )
            )
        }.mapValues { v ->
            jsonMapper.writeValueAsString(v)
        }.to(
            Topics
                .WAIT_FOR_OPPONENT, Produced.with(
                Serdes.UUID(), Serdes
                    .String()
            )
        )

        // open a new game
        createGameStream.map { _, v ->
            val newGame =
                Game(
                    gameId = v.gameId,
                    visibility = v.visibility,
                    creator = v.sessionId,
                    boardSize = v.boardSize
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
            KeyValue(cg.gameId, GameState(board = Board(size = cg.boardSize)))
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

        @Suppress("DEPRECATION") val aggregateAll: KTable<String, GameLobby> =
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
                println("🏟                    GAMLOBBY $json")
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
        val findPublicGameStream: KStream<SessionId, FindPublicGame> =
            streamsBuilder.stream<SessionId, String>(
                Topics.FIND_PUBLIC_GAME,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    jsonMapper.readValue(
                        v,
                        FindPublicGame::class.java
                    )
                }

        val fpgKeyJoiner: KeyValueMapper<SessionId, FindPublicGame, String> =
            KeyValueMapper { _: SessionId,          // left key
                             _: FindPublicGame ->  // left value

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

        val publicGameExists: KStream<SessionId, FindPublicGameLobby> =
            fpgLobbyBranches[0]

        val noPublicGameExists: KStream<SessionId, FindPublicGameLobby> =
            fpgLobbyBranches[1]


        // if someone was looking for a public game and
        // didn't find one
        noPublicGameExists
            .map { _, fo ->
                KeyValue(
                    fo.command.sessionId,
                    // game ID randomly generated here
                    CreateGame(
                        clientId = fo.command.clientId,
                        visibility = Visibility.Public,
                        gameId = UUID.randomUUID(),
                        sessionId = fo.command.sessionId
                    )
                )
            }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.CREATE_GAME,
                Produced.with(Serdes.UUID(), Serdes.String())
            )


        /**
         * The SessionId key is the person finding a game
         * The creator of the game is buried in the game lobby (someGame)
         */
        val popPublicGame: KStream<SessionId, GameLobbyCommand> =
            publicGameExists.map { sid, fl ->
                val someGame =
                    fl.lobby.games.first { g ->
                        g.visibility == Visibility
                            .Public
                    }

                KeyValue(
                    sid,
                    GameLobbyCommand(
                        game = someGame,
                        lobbyCommand = LobbyCommand.Ready
                    )
                )

            }

        popPublicGame.map { finderSessionId, gameCommand ->
            KeyValue(
                gameCommand.game.gameId,
                GameReady(
                    gameCommand.game.gameId,
                    Pair(gameCommand.game.creator, finderSessionId),
                    boardSize = gameCommand.game.boardSize
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
            .map { _, v ->
                KeyValue(v.game.gameId,
                    GameState(board = Board(size = v.game.boardSize)
                )) }
            .mapValues { v -> jsonMapper.writeValueAsString(v) }
            .to(
                Topics.GAME_STATES_CHANGELOG,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

    }

    private fun buildAbandonGameStreams(streamsBuilder: StreamsBuilder,
                                        gameLobby: GlobalKTable<String,
                                                GameLobby>) {
        val sessionDisconnected: KStream<SessionId, SessionDisconnected> =
            streamsBuilder.stream<SessionId, String>(
                Topics.SESSION_DISCONNECTED,
                Consumed.with(Serdes.UUID(), Serdes.String())
            )
                .mapValues { v ->
                    jsonMapper.readValue(
                        v,
                        SessionDisconnected::class.java
                    )
                }

        val kvm: KeyValueMapper<SessionId, SessionDisconnected,
                String> =
            KeyValueMapper { _: SessionId,           // left key
                             _: SessionDisconnected ->  // left value

                // use a trivial join, so that all queries are routed to the same store
                GameLobby.TRIVIAL_KEY
            }

        val valJoiner: ValueJoiner<SessionDisconnected, GameLobby,
                Pair<SessionDisconnected, GameLobby>> =
            ValueJoiner { leftValue:
                          SessionDisconnected,
                          rightValue:
                          GameLobby ->
                Pair(leftValue, rightValue)
            }

        val joined: KStream<SessionId, Pair<SessionDisconnected,GameLobby>> =
            sessionDisconnected.join(gameLobby,kvm,valJoiner)

        val lobbyContainsSessionId: KStream<SessionId,
                Pair<SessionDisconnected, GameLobby>> =
            joined.filter { sessionId, sessionIdLobby  ->
                sessionIdLobby.second.games
                    .map{it.creator}
                    .contains(sessionId)
        }

        val abandonGameCommand: KStream<String, GameLobbyCommand> =
            lobbyContainsSessionId.map{ sessionId, cdgl ->
            val theirGame = cdgl.second.games.first{it.creator == sessionId}
            KeyValue(GameLobby.TRIVIAL_KEY, GameLobbyCommand(
                game = theirGame,
                lobbyCommand = LobbyCommand.Abandon
            ))
        }

        abandonGameCommand
            .mapValues { v ->  jsonMapper.writeValueAsString(v) }
            .to(Topics.GAME_LOBBY_COMMANDS,
                Produced.with(
                    Serdes.String(),
                    Serdes.String())
        )
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


