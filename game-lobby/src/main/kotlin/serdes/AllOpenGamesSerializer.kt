package serdes

import GameLobby
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Serializer

class AllOpenGamesSerializer : Serializer<GameLobby> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, logAgg: GameLobby?): ByteArray? {
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
