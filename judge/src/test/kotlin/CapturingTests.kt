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

        val actual = neighbors(Coord(1, 1), board)

        val expected = setOf(
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(0, 1), Player.WHITE),
            Pair(Coord(2, 1), Player.BLACK),
            Pair(Coord(1, 2), Player.WHITE)
        )

        assertEquals(
            expected, actual, "wrong neighbors"
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

        val actual = neighbors(Coord(0, 1), board)

        val expected = setOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 1), Player.WHITE),
            Pair(Coord(0, 2), Player.WHITE)
        )

        assertEquals(
            expected, actual, "wrong neighbors"
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

        val actual = neighbors(Coord(4, 4), board)

        val expected = setOf(
            Pair(Coord(4, 3), Player.BLACK)
        )

        assertEquals(
            expected, actual, "wrong neighbors"
        )
    }

    @Test
    fun connectedTest() {
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
            Pair(Coord(1, 3), Player.WHITE),
            Pair(Coord(1, 4), Player.WHITE),
            Pair(Coord(1, 5), Player.WHITE),
            Pair(Coord(5, 1), Player.WHITE)
        )

        val board = Board(pieces = pieces)
        val actual = connected(Coord(1, 4), board)

        val expected = pieces.filter { it.value == Player.WHITE }.toList()
            .minus(Pair(Coord(5, 1), Player.WHITE)).map { it.first }
            .toSet()
        assertEquals(expected, actual, "connections incorrect")
    }
}
