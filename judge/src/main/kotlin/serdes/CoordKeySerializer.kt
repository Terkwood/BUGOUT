package serdes

import Coord
import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.databind.JsonSerializer
import com.fasterxml.jackson.databind.SerializerProvider

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
