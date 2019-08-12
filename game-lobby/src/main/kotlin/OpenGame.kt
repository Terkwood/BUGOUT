import serdes.jsonMapper

data class Game (val gameId: GameId, val visibility: Visibility)
data class GameCommand(val game: Game, val command: Command)
enum class Command { Open, Ready }


class AllOpenGames {
    var games: Set<Game> = setOf()

    fun execute(command: GameCommand): AllOpenGames {
        games = when (command.command) {
            Command.Open ->
                games + command.game
            Command.Ready ->
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
