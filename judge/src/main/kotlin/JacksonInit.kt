import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.core.JsonProcessingException
import com.fasterxml.jackson.databind.*
import com.fasterxml.jackson.databind.module.SimpleModule
import com.fasterxml.jackson.databind.util.StdDateFormat
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
    private val regex = Regex("(\\d+)_(\\d+)")
    private fun coordFrom(str: String): Coord? {
        val rm = regex.matchEntire(str)
        val x = rm?.destructured?.component1()
        val y = rm?.destructured?.component2()
        if (x != null && y != null)
            return Coord(x = x.toInt(), y = y.toInt())

        return null
    }

    @Throws(IOException::class, JsonProcessingException::class)
    override fun deserializeKey(key: String, ctxt: DeserializationContext):
            Any? = coordFrom(key)
}

internal class CoordKeySerializer : JsonSerializer<Coord>() {
    override fun serialize(
        value: Coord?,
        gen: JsonGenerator?,
        serializers: SerializerProvider?
    ) {
        if (value != null && gen != null)
            gen.writeFieldName("${value.x}_${value.y}")
    }
}
