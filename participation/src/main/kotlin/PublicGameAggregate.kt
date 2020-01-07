/**
 * An aggregation of public game requests, used to determine when a game is ready,
 * and extract the client IDs for that game.
 * */
class PublicGameAggregate {
    // Keep this property public - it's exposed to kafka!
    @Suppress("MemberVisibilityCanBePrivate")
    val requests = hashMapOf<GameId, List<ClientId>>()

    fun add(gameId: GameId, findPublicGame: FindPublicGame) : PublicGameAggregate {
        println("add $gameId $findPublicGame")
        println("existing ${requests[gameId]}")

        requests[gameId] =
            listOf(findPublicGame.clientId) + requests[gameId].orEmpty()

        println("updated ${requests[gameId]}")

        return this
    }

    fun ready(gameId: GameId): Boolean = requests.containsKey(gameId) && requests[gameId]?.size == 2

    fun clients(gameId: GameId) : List<ClientId> = requests[gameId].orEmpty()
}