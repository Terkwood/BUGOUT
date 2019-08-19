import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.streams.*
import org.apache.kafka.streams.kstream.*
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
                            jsonMapper.readValue(
                                    v,
                                    ClientGameReady::class.java
                            )
                        }

        val clientGameColorPrefKVM: KeyValueMapper<ClientIdKey,
                ClientGameReady, ClientIdKey> =
                // left key, left value
                KeyValueMapper { clientId: ClientIdKey,
                                 _: ClientGameReady ->
                    clientId
                }

        val clientGameColorPrefVJ: ValueJoiner<ClientGameReady,
                ChooseColorPref, ClientGameColorPref> =
                ValueJoiner { leftValue: ClientGameReady,
                              rightValue: ChooseColorPref ->
                    ClientGameColorPref(leftValue.clientId, leftValue.gameId,
                            rightValue.colorPref)
                }

        val clientGameColorPref: KStream<ClientIdKey, ClientGameColorPref> =
                clientGameReady.join(chooseColorPref,clientGameColorPrefVJ,
                        null)

        return streamsBuilder.build()
    }
}
