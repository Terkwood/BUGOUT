import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.*
import org.apache.kafka.streams.kstream.*
import serdes.jsonMapper
import java.util.*
import java.util.concurrent.TimeUnit

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
        props[StreamsConfig.APPLICATION_ID_CONFIG] = "bugout-color-chooser"
        props[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = "exactly_once"

        val streams = KafkaStreams(topology, props)
        streams.start()
    }


    fun build(): Topology {
        val streamsBuilder = StreamsBuilder()

        val chooseColorPref: KStream<ClientIdKey, ChooseColorPref> =
                streamsBuilder.stream<UUID, String>(
                        Topics.CHOOSE_COLOR_PREF,
                        Consumed.with(Serdes.UUID(), Serdes.String())
                )
                        .mapValues { v ->
                            println("pref chosen $v")
                            jsonMapper.readValue(
                                    v,
                                    ChooseColorPref::class.java
                            )
                        }

        val gameReady: KStream<GameIdKey, GameReady> =
                streamsBuilder.stream<UUID, String>(
                        Topics.GAME_READY,
                        Consumed.with(Serdes.UUID(), Serdes.String())
                )
                        .mapValues { v ->
                            jsonMapper.readValue(
                                    v,
                                    GameReady::class.java
                            )
                        }

        // generate  a ClientGameReady event for the first client
        gameReady
                .map { _, gr ->
                    KeyValue(gr.clients.first.underlying,
                            ClientGameReady(gr.clients.first, gr.gameId))
                }
                .mapValues { cgr -> jsonMapper.writeValueAsString(cgr) }
                .to(Topics.CLIENT_GAME_READY,
                        Produced.with(Serdes.UUID(), Serdes.String()))

        // generate a ClientGameReady event for the second client
        gameReady
                .map { _, gr ->
                    KeyValue(gr.clients.second.underlying,
                            ClientGameReady(gr.clients.second, gr.gameId))
                }
                .mapValues { cgr -> jsonMapper.writeValueAsString(cgr) }
                .to(Topics.CLIENT_GAME_READY,
                        Produced.with(Serdes.UUID(), Serdes.String()))


        val clientGameReady: KStream<ClientIdKey, ClientGameReady> =
                streamsBuilder.stream<UUID, String>(
                        Topics.CLIENT_GAME_READY,
                        Consumed.with(Serdes.UUID(), Serdes.String())
                )
                        .mapValues { v ->
                            println("client game ready $v")
                            jsonMapper.readValue(
                                    v,
                                    ClientGameReady::class.java
                            )
                        }

        val prefJoiner: ValueJoiner<ClientGameReady,
                ChooseColorPref, ClientGameColorPref> =
                ValueJoiner { leftValue: ClientGameReady,
                              rightValue: ChooseColorPref ->
                    ClientGameColorPref(leftValue.clientId, leftValue.gameId,
                            rightValue.colorPref)
                }


        // TODO serialization fails here
        // TODO join window sanity
        val clientGameColorPref: KStream<ClientIdKey, ClientGameColorPref> =
                clientGameReady.join(chooseColorPref, prefJoiner,
                        JoinWindows.of(TimeUnit.DAYS.toMillis(1000)))

        /*
        clientGameColorPref
                .map { _, gcp ->
                    KeyValue(gcp.gameId.underlying,
                            gcp)
                }
                .mapValues { v -> jsonMapper.writeValueAsString(v) }
                .to(Topics.AGGREGATE_COLOR_PREF,
                        Produced.with(Serdes.UUID(), Serdes.String()))
*/
        return streamsBuilder.build()
    }
}
