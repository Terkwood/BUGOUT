import com.fasterxml.jackson.annotation.JsonIgnoreProperties

data class GameState(
    val board: Board = Board(),
    val turn: Int = 0,
    val playerUp: Player = Player.BLACK
)

@JsonIgnoreProperties(value = ["board", "captures", "playerUp", "moves"])
data class GameStateTurnOnly(val turn: Int)

data class GameStateLobby(val gameState: GameStateTurnOnly, val lobby: GameLobby)