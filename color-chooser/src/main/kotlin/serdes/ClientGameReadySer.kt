package serdes

import ClientGameReady
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Serializer

class ClientGameReadySer : Serializer<ClientGameReady> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, data: ClientGameReady?): ByteArray? {
        if (data == null) {
            return null
        }

        try {
            return data.asByteArray()
        } catch (e: RuntimeException) {
            throw SerializationException("Error serializing value", e)
        }

    }

}