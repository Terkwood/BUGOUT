package serdes

import ClientGameColorPref
import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.Serializer
import java.io.*


class ListPrefSer : Serializer<List<ClientGameColorPref>> {

    private val inner = BoringDataSer<ClientGameColorPref>()

    override fun serialize(topic: String, data: List<ClientGameColorPref>?): ByteArray? {
        if (data == null) {
            return null
        }

        val size = data.size
        val baos = ByteArrayOutputStream()
        val dos = DataOutputStream(baos)
        val iterator = data.iterator()
        try {
            dos.writeInt(size)
            while( iterator.hasNext()) {
                val bytes = inner.serialize(topic, iterator.next())
                if (bytes != null) {
                    dos.writeInt(bytes.size)
                    dos.write(bytes)
                }
            }
        } catch (e: IOException) {
            throw RuntimeException("Cannot serialize list", e)
        }

        return baos.toByteArray()
    }

}

class ListPrefDes : Deserializer<List<ClientGameColorPref>> {
    private val inner = ClientGameColorPrefDes()

    override fun configure(configs: Map<String, *>, isKey: Boolean) {}

    override fun close() {}

    override fun deserialize(topic: String, bytes: ByteArray?): List<ClientGameColorPref>? {
        if (bytes == null || bytes.isEmpty()) {
            return null
        }

        val arrayList = arrayListOf<ClientGameColorPref>()
        val dataInputStream = DataInputStream(ByteArrayInputStream(bytes))

        try {
            val records = dataInputStream.readInt()
            for (i in 0..records) {
                val valueBytes: ByteArray = ByteArray(dataInputStream.readInt())
                dataInputStream.read(valueBytes)
                val d = inner.deserialize(topic, valueBytes)
                if (d != null) {
                    arrayList.add(d)
                } else {
                    throw IOException("greetings")
                }
            }
        } catch(e: IOException) {
            throw RuntimeException("Cannot deserialize list")
        }

        return arrayList.toList()
    }

}