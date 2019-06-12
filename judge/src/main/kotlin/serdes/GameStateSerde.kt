package serdes

import GameState
import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serde
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.Serializer

private val gameBoardSerializer: Serializer<GameState> =
    GameStateSerializer()

private val gameBoardDeserializer: Deserializer<GameState> =
    GameStateDeserializer()

val gameBoardSerde: Serde<GameState> =
    Serdes.serdeFrom(gameBoardSerializer, gameBoardDeserializer)
