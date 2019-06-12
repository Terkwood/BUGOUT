import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class CapturingTests {
    @Test
    fun correctNeighbors() {
        val pieces: MutableMap<Coord, Player> = hashMapOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(2, 0), Player.BLACK),
            Pair(Coord(0, 1), Player.WHITE),
            Pair(Coord(1, 1), Player.WHITE),
            Pair(Coord(2, 1), Player.BLACK),
            Pair(Coord(0, 2), Player.WHITE),
            Pair(Coord(1, 2), Player.WHITE),
            Pair(Coord(2, 2), Player.WHITE),
            Pair(Coord(4, 3), Player.BLACK),
            Pair(Coord(1, 5), Player.WHITE)
        )

        val board = Board(pieces = pieces)
        val expected = setOf(
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(0, 1), Player.WHITE),
            Pair(Coord(2, 1), Player.BLACK),
            Pair(Coord(1, 2), Player.WHITE)
        )
        assertEquals(
            expected, neighbors(Coord(1, 1), board), "wrong neighbors"
        )
    }

    @Test
    fun edgeNeighbors() {
        val pieces: MutableMap<Coord, Player> = hashMapOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(2, 0), Player.BLACK),
            Pair(Coord(0, 1), Player.WHITE),
            Pair(Coord(1, 1), Player.WHITE),
            Pair(Coord(2, 1), Player.BLACK),
            Pair(Coord(0, 2), Player.WHITE),
            Pair(Coord(1, 2), Player.WHITE),
            Pair(Coord(2, 2), Player.WHITE),
            Pair(Coord(4, 3), Player.BLACK),
            Pair(Coord(1, 5), Player.WHITE)
        )

        val board = Board(pieces = pieces)
        val expected = setOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 1), Player.WHITE),
            Pair(Coord(0, 2), Player.WHITE)
        )
        assertEquals(
            expected, neighbors(Coord(0, 1), board), "wrong neighbors"
        )
    }


    @Test
    fun noEmptyNeighbors() {
        val pieces: MutableMap<Coord, Player> = hashMapOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(2, 0), Player.BLACK),
            Pair(Coord(0, 1), Player.WHITE),
            Pair(Coord(1, 1), Player.WHITE),
            Pair(Coord(2, 1), Player.BLACK),
            Pair(Coord(0, 2), Player.WHITE),
            Pair(Coord(1, 2), Player.WHITE),
            Pair(Coord(2, 2), Player.WHITE),
            Pair(Coord(4, 3), Player.BLACK),
            Pair(Coord(1, 5), Player.WHITE)
        )

        val board = Board(pieces = pieces)
        val expected = setOf(
            Pair(Coord(4, 3), Player.BLACK)
        )
        assertEquals(
            expected, neighbors(Coord(4, 4), board), "wrong neighbors"
        )
    }
}
