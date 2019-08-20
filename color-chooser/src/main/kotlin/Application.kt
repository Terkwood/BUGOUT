import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.utils.Bytes
import org.apache.kafka.streams.*
import org.apache.kafka.streams.kstream.*
import org.apache.kafka.streams.state.KeyValueStore
import serdes.*
import java.time.temporal.ChronoUnit
import java.util.*

fun main() {
    TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
    Application("kafka:9092").process()
}

const val REQUIRED_PREFS = 2

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

        buildGameColorPref(streamsBuilder)

        val aggregated = aggregateColorPrefs(streamsBuilder)

        val readyToChoose: KStream<GameId, AggregatedPrefs> =
            aggregated.toStream().filter { _, agg -> agg.prefs.size == REQUIRED_PREFS }

        readyToChoose.mapValues { agg ->
            ColorsChosen.resolve(agg.prefs[0], agg.prefs[1])
        }.mapValues { v ->
            println("hello from $v")
            jsonMapper.writeValueAsString(v)
        }.to(Topics.COLORS_CHOSEN, Produced.with(Serdes.UUID(), Serdes.String()))

        return streamsBuilder.build()
    }


    private fun buildGameColorPref(streamsBuilder: StreamsBuilder) {
        val chooseColorPref: KStream<ClientId, ChooseColorPref> =
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

        val gameReady: KStream<GameId, GameReady> =
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
                KeyValue(
                    gr.clients.first,
                    ClientGameReady(gr.clients.first, gr.gameId)
                )
            }
            .mapValues { cgr -> jsonMapper.writeValueAsString(cgr) }
            .to(
                Topics.CLIENT_GAME_READY,
                Produced.with(Serdes.UUID(), Serdes.String())
            )

        // generate a ClientGameReady event for the second client
        gameReady
            .map { _, gr ->
                KeyValue(
                    gr.clients.second,
                    ClientGameReady(gr.clients.second, gr.gameId)
                )
            }
            .mapValues { cgr -> jsonMapper.writeValueAsString(cgr) }
            .to(
                Topics.CLIENT_GAME_READY,
                Produced.with(Serdes.UUID(), Serdes.String())
            )


        val clientGameReady: KStream<ClientId, ClientGameReady> =
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

        val prefJoiner: ValueJoiner<ClientGameReady,
                ChooseColorPref, ClientGameColorPref> =
            ValueJoiner { leftValue: ClientGameReady,
                          rightValue: ChooseColorPref ->
                ClientGameColorPref(
                    leftValue.clientId, leftValue.gameId,
                    rightValue.colorPref
                )
            }


        val clientGameColorPref: KStream<ClientId, ClientGameColorPref> =
            clientGameReady.join(
                chooseColorPref, prefJoiner,
                JoinWindows.of(ChronoUnit.YEARS.duration),
                Joined.with(
                    Serdes.UUID(),
                    Serdes.serdeFrom(ClientGameReadySer(), ClientGameReadyDes()),
                    Serdes.serdeFrom(ChooseColorPrefSer(), ChooseColorPrefDes())
                )
            )

        val gameColorPref = clientGameColorPref
            .map { _, gcp ->
                KeyValue(
                    gcp.gameId,
                    gcp
                )
            }


        // mainly a checkpoint for testing
        gameColorPref.mapValues { v -> jsonMapper.writeValueAsString(v) }.to(
            Topics.GAME_COLOR_PREF,
            Produced.with(Serdes.UUID(), Serdes.String())
        )

    }


    private fun aggregateColorPrefs(
        streamsBuilder: StreamsBuilder
    ): KTable<GameId, AggregatedPrefs> =
        streamsBuilder
            .stream<UUID, String>(
                Topics.GAME_COLOR_PREF, Consumed.with(Serdes.UUID(), Serdes.String())
            ).groupByKey(Serialized.with(Serdes.UUID(), Serdes.String()))
            .aggregate({ AggregatedPrefs() }, { _, p, allPrefs ->
                allPrefs.add(jsonMapper.readValue(p, ClientGameColorPref::class.java))
                allPrefs
            }, Materialized.`as`<UUID, AggregatedPrefs, KeyValueStore<Bytes, ByteArray>>(
                Topics.REDUCE_COLOR_PREFS_STORE
            )
                .withKeySerde(Serdes.UUID())
                .withValueSerde(Serdes.serdeFrom(AggPrefSer(), AggPrefDes()))
            )
}
