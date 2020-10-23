import serdes.jsonMapper

/**
 * Represents a point in time for a game
 */
class GameState(boardSize: Int = FULL_BOARD_SIZE) {
    val board = Board(size = boardSize)

    var captures = Captures()

    var turn: Int = 1

    var playerUp: Player = Player.BLACK

    val moves: MutableList<MoveMade> = mutableListOf()

    fun add(ev: MoveMade): GameState {
        moves.add(ev)

        if (ev.coord != null) {
            board.pieces[ev.coord] = ev.player
            ev.captured.forEach { coord ->
                board.pieces.remove(coord)
                when (ev.player) {
                    Player.BLACK -> captures.black = captures.black + 1
                    Player.WHITE -> captures.white = captures.white + 1
                }
            }
        }

        turn++

        playerUp = when (playerUp) {
            Player.BLACK -> Player.WHITE
            Player.WHITE -> Player.BLACK
        }

        return this
    }

    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}
