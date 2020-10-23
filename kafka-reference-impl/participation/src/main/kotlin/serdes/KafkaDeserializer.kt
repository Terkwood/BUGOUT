package serdes

import com.fasterxml.jackson.core.type.TypeReference
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class KafkaDeserializer<A>(t: TypeReference<A>) : Deserializer<A> {
    private val typeReference = t

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): A? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, typeReference)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}
