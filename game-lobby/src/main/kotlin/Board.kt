const val FULL_SIZE_BOARD = 19
data class Board(val pieces: Map<Nothing, Player> = mapOf(),
                 val size: Int = FULL_SIZE_BOARD)