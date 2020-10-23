package serdes

import Coord
import com.fasterxml.jackson.core.JsonProcessingException
import com.fasterxml.jackson.databind.DeserializationContext
import com.fasterxml.jackson.databind.KeyDeserializer
import java.io.IOException

internal class CoordKeyDeserializer :
    KeyDeserializer() {
    private val regex = Regex("(\\d+),(\\d+)")
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

