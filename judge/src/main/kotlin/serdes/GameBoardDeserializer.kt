package serdes

import GameBoard
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class GameBoardDeserializer : Deserializer<GameBoard> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): GameBoard? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, GameBoard::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}
