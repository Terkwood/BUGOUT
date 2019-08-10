import serdes.jsonMapper

data class GameState(
    val board: Board = Board(),
    val captures: Captures = Captures(),
    val turn: Int = 1,
    val playerUp: Player = Player.BLACK,
    val moves: List<MoveMade> = listOf()
) {

    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}
