package serdes

import ClientGameColorPref
import com.fasterxml.jackson.core.type.TypeReference

import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serializer


class ListPrefSer : Serializer<List<ClientGameColorPref>> {

    override fun serialize(topic: String, data: List<ClientGameColorPref>?): ByteArray? {
        if (data == null) {
            return null
        }

        try {
            return data.asByteArray()
        } catch (e: java.lang.RuntimeException) {
            throw SerializationException("Error serializing", e)
        }
    }

}

fun List<ClientGameColorPref>.asByteArray(): ByteArray {
    return jsonMapper.writeValueAsBytes(this)
}


class ListPrefDes : Deserializer<List<ClientGameColorPref>> {

    private class TR: TypeReference<List<ClientGameColorPref>>() {}

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): List<ClientGameColorPref>? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes,TR())
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}