package serdes

import AllOpenGames
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Serializer

class AllOpenGamesSerializer : Serializer<AllOpenGames> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, logAgg: AllOpenGames?): ByteArray? {
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
