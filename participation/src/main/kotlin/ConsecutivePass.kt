enum class Move { PlaceStone, Pass }

class ConsecutivePass {
    // Must be public so it can be serialized
    var turns = listOf<Move>()

    fun track(move: Move) : ConsecutivePass {
        turns = listOf(move) + turns.take(1)
        return this
    }

    fun happened(): Boolean =
        turns.size == 2 && turns[0] == Move.Pass && turns[1] == Move.Pass
}