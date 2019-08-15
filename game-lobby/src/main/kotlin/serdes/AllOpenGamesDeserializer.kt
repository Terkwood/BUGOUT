package serdes

import GameLobby
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class AllOpenGamesDeserializer : Deserializer<GameLobby> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): GameLobby? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, GameLobby::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}
