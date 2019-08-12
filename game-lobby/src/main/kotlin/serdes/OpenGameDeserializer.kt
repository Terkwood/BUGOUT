package serdes

import OpenGame
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class OpenGameDeserializer : Deserializer<OpenGame> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): OpenGame? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, OpenGame::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}
