package serdes


import ChooseColorPref
import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serializer

class ChooseColorPrefDes : Deserializer<ChooseColorPref> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): ChooseColorPref? {
        if (bytes == null) {
            return null
        }

        try {
            return jsonMapper.readValue(bytes, ChooseColorPref::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("Error deserializing value", e)
        }

    }

}


class ChooseColorPrefSer : Serializer<ChooseColorPref> {

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun serialize(topic: String, logAgg: ChooseColorPref?): ByteArray? {
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
