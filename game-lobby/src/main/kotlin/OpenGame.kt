import serdes.jsonMapper

data class OpenGame(val gameId: GameId, val visibility: Visibility)


class AllOpenGames {
    var games: Set<OpenGame> = setOf()


    fun add(openGame: OpenGame): AllOpenGames {
        games += openGame

        return this
    }

    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }


    companion object  {
        val TOPIC_KEY: Short = 0

    }
}

data class FindPublicGameAllOpenGames(val command: FindPublicGame, val store: AllOpenGames)
