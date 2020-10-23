enum class Move { PlaceStone, Pass }

class ConsecutivePass {
    // Must be public so it can be serialized
    @Suppress("MemberVisibilityCanBePrivate")
    val turns = hashMapOf<GameId, List<Move>>()

    fun track(gameId: GameId, move: Move) : ConsecutivePass {
        turns[gameId] = listOf(move) + turns[gameId].orEmpty().take(1)
        return this
    }

    fun happenedIn(gameId: GameId): Boolean =
        turns[gameId]?.size == 2 && turns[gameId]?.get(0) == Move.Pass && turns[gameId]?.get(1) == Move.Pass
}