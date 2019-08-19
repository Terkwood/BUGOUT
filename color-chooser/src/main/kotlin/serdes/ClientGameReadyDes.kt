package serdes


import ClientGameReady
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer

class ClientGameReadyDes : Deserializer<ClientGameReady> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): ClientGameReady? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, ClientGameReady::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}


