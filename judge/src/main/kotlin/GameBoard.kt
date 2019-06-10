class GameBoard {
    private val _board: MutableMap<Coord, Player> = HashMap()
    fun heedlessAdd(move: MakeMoveCmd): GameBoard {
        if (!_board.containsKey(move.coord))
            _board[move.coord] = move.player

        return this
    }
}
