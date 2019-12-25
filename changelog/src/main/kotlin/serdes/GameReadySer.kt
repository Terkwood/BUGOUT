package serdes

import GameReady
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Serializer

class GameReadySer : Serializer<GameReady> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, logAgg: GameReady?): ByteArray? {
        if (logAgg == null) {
            return null
        }

        try {
            return logAgg.asByteArray()
        } catch (e: RuntimeException) {
            throw SerializationException("Error serializing value", e)
        }

    }

}
