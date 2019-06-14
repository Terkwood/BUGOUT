fun MoveCommandGameState.isValid(): Boolean {
    val move = this.moveCmd
    val game = this.gameState
    val correctPlayer = move.player == game.playerUp
    val coord = move.coord
    val passing = coord == null
    val coordExists: Boolean by lazy {
        if (coord != null) {
            val size = game.board.size
            val (x, y) = coord
            size > x && size > y && x >= 0 && y >= 0
        } else false
    }
    val validCoord: Boolean by lazy {
        coordExists && game.board.pieces[move.coord] == null
    }

    return correctPlayer && (passing || validCoord)
}
