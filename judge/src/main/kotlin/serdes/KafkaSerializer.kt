package serdes

import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Serializer

class KafkaSerializer<A> : Serializer<A> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, a: A?): ByteArray? {
        if (a == null) {
            return null
        }

        try {
            return jsonMapper.writeValueAsBytes(a)
        } catch (e: RuntimeException) {
            throw SerializationException("Error serializing value", e)
        }
    }
}
