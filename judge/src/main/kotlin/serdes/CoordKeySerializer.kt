package serdes

import Coord
import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.databind.JsonSerializer
import com.fasterxml.jackson.databind.SerializerProvider

// Provide a JSON-prettified key string format for our Coord class
internal class CoordKeySerializer : JsonSerializer<Coord>() {
    override fun serialize(
        value: Coord?,
        gen: JsonGenerator?,
        serializers: SerializerProvider?
    ) {
        if (value != null && gen != null)
            gen.writeFieldName("${value.x},${value.y}")
    }
}
