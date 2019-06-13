import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class CapturingTests {
    @Test
    fun correctNeighborPieces() {
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

        val actual = neighborPieces(Coord(1, 1), board)

        val expected = setOf(
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(0, 1), Player.WHITE),
            Pair(Coord(2, 1), Player.BLACK),
            Pair(Coord(1, 2), Player.WHITE)
        )

        assertEquals(
            expected, actual, "wrong neighborPieces"
        )
    }

    @Test
    fun edgeNeighborPieces() {
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

        val actual = neighborPieces(Coord(0, 1), board)

        val expected = setOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 1), Player.WHITE),
            Pair(Coord(0, 2), Player.WHITE)
        )

        assertEquals(
            expected, actual, "wrong neighborPieces"
        )
    }


    @Test
    fun noEmptyNeighborPieces() {
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

        val actual = neighborPieces(Coord(4, 4), board)

        val expected = setOf(
            Pair(Coord(4, 3), Player.BLACK)
        )

        assertEquals(
            expected, actual, "wrong neighborPieces"
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

    @Test
    fun connectionsEmpty() {
        val pieces: MutableMap<Coord, Player> = hashMapOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(2, 0), Player.BLACK)
        )
        val board = Board(pieces)
        val actual = connected(Coord(4, 4), board)
        val expected: Set<Coord> = setOf()
        assertEquals(expected, actual, "non empty isn't so good")
    }

    @Test
    fun basicLiberties() {
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

        val board = Board(pieces)

        val actual = liberties(Coord(1, 3), board)

        val expected = setOf(
            Coord(3, 2),
            Coord(0, 3),
            Coord(2, 3),
            Coord(0, 4),
            Coord(2, 4),
            Coord(0, 5),
            Coord(2, 5),
            Coord(1, 6)
        )

        assertEquals(expected, actual, "wrong freedoms")
    }
}
