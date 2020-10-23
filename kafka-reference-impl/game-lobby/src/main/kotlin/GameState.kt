data class GameState(
    val board: Board = Board(),
    val turn: Int = 1,
    val playerUp: Player = Player.BLACK
)
