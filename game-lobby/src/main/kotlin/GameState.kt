data class GameState(
    val board: Board = Board(),
    val turn: Int = 0,
    val playerUp: Player = Player.BLACK
)

data class GameStateTurnOnly(val turn: Int)
