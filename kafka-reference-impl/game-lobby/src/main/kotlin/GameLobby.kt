import serdes.jsonMapper

data class Game(
    val gameId: GameId,
    val visibility: Visibility,
    val creator: SessionId,
    val boardSize: Int = FULL_SIZE_BOARD
)

data class GameLobbyCommand(val game: Game, val lobbyCommand: LobbyCommand)
enum class LobbyCommand { Open, Ready, Abandon }


class GameLobby {
    var games: List<Game> = listOf()

    fun execute(command: GameLobbyCommand): GameLobby {
        games = when (command.lobbyCommand) {
            LobbyCommand.Open ->
                games + command.game
            LobbyCommand.Ready ->
                games - command.game
            LobbyCommand.Abandon ->
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

/**
 * Joiner class
 */
data class FindPublicGameLobby(
    val command: FindPublicGame,
    val lobby: GameLobby
)

/**
 * Joiner class
 */
data class JoinPrivateGameLobby(
    val command: JoinPrivateGame,
    val lobby: GameLobby
)
