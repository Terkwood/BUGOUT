import serdes.jsonMapper

data class Game(
    val gameId: GameId,
    val visibility: Visibility,
    val creator: ClientId
)

data class GameLobbyCommand(val game: Game, val lobbyCommand: LobbyCommand)
enum class LobbyCommand { Open, Ready }


class GameLobby {
    var games: List<Game> = listOf()

    fun execute(command: GameLobbyCommand): GameLobby {
        games = when (command.lobbyCommand) {
            LobbyCommand.Open ->
                games + command.game
            LobbyCommand.Ready ->
                games - command.game
        }

        return this
    }

    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }


    companion object {
        /**
         * Trivial key for kafka join
         */
        const val TRIVIAL_KEY: String = "ALL"
    }
}

data class FindPublicGameAllOpenGames(
    val command: FindPublicGame,
    val store: GameLobby
)
