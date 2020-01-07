class PublicGameAggregate {
    private val requests = hashMapOf<GameId, List<FindPublicGame>>()

    fun add(gameId: GameId, findPublicGame: FindPublicGame) : PublicGameAggregate {
        requests.merge(gameId, listOf(findPublicGame)) {t,u -> t.plus(u) }

        return this
    }
}