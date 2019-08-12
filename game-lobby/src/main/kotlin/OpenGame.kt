import serdes.jsonMapper

data class Game (val gameId: GameId, val visibility: Visibility)
data class OpenGameCommand(val game: Game, val action: OpenGameAction)
enum class OpenGameAction { Add, Remove }


class AllOpenGames {
    var games: Set<Game> = setOf()

    fun update(command: OpenGameCommand): AllOpenGames {
        games = when (command.action) {
            OpenGameAction.Add ->
                games + command.game
            OpenGameAction.Remove ->
                games - command.game
        }

        return this
    }

    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }


    companion object  {
        /**
         * Trivial key for kafka join
         */
        const val TOPIC_KEY: Short = 0
    }
}

data class FindPublicGameAllOpenGames(val command: FindPublicGame, val store: AllOpenGames)
