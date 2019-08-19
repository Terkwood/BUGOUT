package serdes

import ClientGameColorPref
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serializer

class BoringDataSer<T> : Serializer<T> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, data: T?): ByteArray? {
        if (data == null) {
            return null
        }

        try {
            return jsonMapper.writeValueAsBytes(data)
        } catch (e: RuntimeException) {
            throw SerializationException("Error serializing value", e)
        }

    }

}

class ClientGameColorPrefDes : Deserializer<ClientGameColorPref> {

    override fun deserialize(topic: String?, bytes: ByteArray?): ClientGameColorPref? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, ClientGameColorPref::class.java)
        } catch (e: Exception) {
            throw SerializationException(e)
        }

    }

}