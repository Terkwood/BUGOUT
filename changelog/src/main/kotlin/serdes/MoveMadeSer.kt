package serdes

import MoveMade
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Serializer

class MoveMadeSer : Serializer<MoveMade> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, logAgg: MoveMade?): ByteArray? {
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
