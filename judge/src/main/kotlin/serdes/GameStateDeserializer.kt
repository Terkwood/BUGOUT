package serdes

import GameState
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class GameStateDeserializer : Deserializer<GameState> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): GameState? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, GameState::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}
