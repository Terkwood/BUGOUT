import org.apache.kafka.common.serialization.Deserializer
import org.apache.kafka.common.serialization.Serde
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.Serializer

class GameBoard {
    // the pieces are tracked in order
    val pieces: MutableMap<Coord, Placement> = HashMap()

    var captures = Captures()

    var turn: Int = 1

    val passedTurns: MutableList<Pair<Int, Player>> = ArrayList()

    fun add(ev: MoveMadeEv): GameBoard {
        if (ev.coord != null) {
            pieces[ev.coord] = Placement(ev.player, turn)
            updateCaptures(ev.player, ev.captured)
        } else {
            // passing
            passedTurns.add(Pair(this.turn, ev.player))
        }

        turn++

        return this
    }

    fun isValid(ev: MakeMoveCmd): Boolean =
        ev.coord == null || TODO()

    private fun updateCaptures(player: Player, captures: List<Coord>) {
        when (player) {
            Player.BLACK -> this.captures = Captures(
                black = this.captures
                    .black + captures.size, white = this.captures.white
            )
            Player.WHITE -> this.captures = Captures(
                black = this.captures
                    .black, white = this.captures
                    .white + captures.size
            )
        }
    }

    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}

private val gameBoardSerializer: Serializer<GameBoard> =
    GameBoardSerializer()

private val gameBoardDeserializer: Deserializer<GameBoard> =
    GameBoardDeserializer()

val gameBoardSerde: Serde<GameBoard> =
    Serdes.serdeFrom(gameBoardSerializer, gameBoardDeserializer)
