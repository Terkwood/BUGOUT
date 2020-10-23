package serdes

import AggregatedPrefs
import com.fasterxml.jackson.core.type.TypeReference

import org.apache.kafka.common.errors.SerializationException
import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serializer
import java.lang.RuntimeException

class AggPrefSer : Serializer<AggregatedPrefs> {
    override fun serialize(topic: String?, data: AggregatedPrefs?): ByteArray? {
        if (data == null) {return null}

        try {return data.asByteArray()}catch(e: RuntimeException) {
            throw SerializationException("error serializing aggregated prefs", e)
        }
    }
}

class AggPrefDes : Deserializer<AggregatedPrefs> {
    override fun deserialize(topic: String?, data: ByteArray?): AggregatedPrefs? {
        if (data == null) {return null}

        try {
            return jsonMapper.readValue(data, AggregatedPrefs::class.java)
        } catch (e: RuntimeException) {
            throw SerializationException("error deserializing aggregated prefs", e)
        }
    }

}