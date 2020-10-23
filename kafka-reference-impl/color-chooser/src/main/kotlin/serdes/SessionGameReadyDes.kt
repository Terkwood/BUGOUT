package serdes


import SessionGameReady
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class SessionGameReadyDes : Deserializer<SessionGameReady> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): SessionGameReady? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, SessionGameReady::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}


