package serdes

import OpenGame
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Serializer

class OpenGameSerializer : Serializer<OpenGame> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, logAgg: OpenGame?): ByteArray? {
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
