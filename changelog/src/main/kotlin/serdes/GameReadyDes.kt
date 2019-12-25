package serdes

import GameReady
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class GameReadyDes : Deserializer<GameReady> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): GameReady? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, GameReady::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}
