package serdes

import MoveMade
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class MoveMadeDes : Deserializer<MoveMade> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): MoveMade? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, MoveMade::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}
