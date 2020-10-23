package serdes

import Coord
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.SerializationFeature
import com.fasterxml.jackson.databind.module.SimpleModule
import com.fasterxml.jackson.databind.util.StdDateFormat
import com.fasterxml.jackson.module.kotlin.registerKotlinModule


val jsonMapper = ObjectMapper().apply {
    registerKotlinModule()
    disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS)
    dateFormat = StdDateFormat()

    // allow using Coord as key in the GameState pieces field
    val simpleModule = SimpleModule()
    simpleModule.addKeyDeserializer(
        Coord::class.java,
        CoordKeyDes()
    )
    simpleModule.addKeySerializer(
        Coord::class.java,
        CoordKeySer()
    )
    registerModule(simpleModule)
}

