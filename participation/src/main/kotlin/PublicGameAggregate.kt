class PublicGameAggregate {
    private val requests = hashMapOf<GameId, Set<FindPublicGame>>()

    fun add(gameId: GameId, findPublicGame: FindPublicGame) : PublicGameAggregate {
        requests.merge(gameId, setOf(findPublicGame)) {t,u -> t.plus(u) }

        return this
    }

    fun ready(): Boolean = requests.size == 2
}