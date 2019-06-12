import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.core.JsonProcessingException
import com.fasterxml.jackson.databind.*
import com.fasterxml.jackson.databind.module.SimpleModule
import com.fasterxml.jackson.databind.util.StdDateFormat
import com.fasterxml.jackson.module.kotlin.readValue
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import java.io.IOException


val jsonMapper = ObjectMapper().apply {
    registerKotlinModule()
    disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS)
    dateFormat = StdDateFormat()

    // allow using Coord as key in the GameBoard pieces field
    val simpleModule = SimpleModule()
    simpleModule.addKeyDeserializer(
        Coord::class.java,
        CoordKeyDeserializer()
    )
    simpleModule.addKeySerializer(
        Coord::class.java,
        CoordKeySerializer()
    )
    registerModule(simpleModule)
}

internal class CoordKeyDeserializer : KeyDeserializer() {
    @Throws(IOException::class, JsonProcessingException::class)
    override fun deserializeKey(key: String, ctxt: DeserializationContext):
            Any? = jsonMapper.readValue<Coord>(key)
}

internal class CoordKeySerializer : JsonSerializer<Coord>() {
    override fun serialize(
        value: Coord?,
        gen: JsonGenerator?,
        serializers: SerializerProvider?
    ) {
        if (value != null && gen != null)
            gen.writeFieldName(jsonMapper.writeValueAsString(value))
    }
}
