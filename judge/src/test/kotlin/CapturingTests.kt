import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class CapturingTests {
    @Test
    fun correctNeighborPieces() {
        val pieces: Map<Coord, Player> = hashMapOf(
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
        val pieces: Map<Coord, Player> = hashMapOf(
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
        val pieces: Map<Coord, Player> = hashMapOf(
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
        val pieces: Map<Coord, Player> = hashMapOf(
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
    fun soConnectedTest() {
        val pieces: Map<Coord, Player> = hashMapOf(
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
        val actual = connected(Coord(1, 0), board)

        val expected = pieces.filter { it.value == Player.BLACK }.toList()
            .minus(Pair(Coord(4, 3), Player.BLACK)).map { it.first }
            .toSet()
        assertEquals(expected, actual, "connections incorrect")
    }

    @Test
    fun connectionsEmpty() {
        val pieces: Map<Coord, Player> = hashMapOf(
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
        val pieces: Map<Coord, Player> = hashMapOf(
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

    @Test
    fun moreFreedoms() {
        val pieces: Map<Coord, Player> = hashMapOf(
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

        val actual = liberties(Coord(0, 0), board)

        val expected = setOf(
            Coord(3, 0),
            Coord(3, 1)
        )

        assertEquals(expected, actual, "freedom broken")
    }

    @Test
    fun takeOverTheWorld() {
        val pieces: Map<Coord, Player> = hashMapOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(2, 0), Player.BLACK),
            Pair(Coord(3, 0), Player.WHITE),
            Pair(Coord(0, 1), Player.WHITE),
            Pair(Coord(1, 1), Player.WHITE),
            Pair(Coord(2, 1), Player.BLACK),
            Pair(Coord(0, 2), Player.WHITE),
            Pair(Coord(1, 2), Player.WHITE),
            Pair(Coord(2, 2), Player.WHITE),
            Pair(Coord(4, 3), Player.BLACK),
            Pair(Coord(1, 5), Player.WHITE)
        )

        val board = Board(pieces)

        val actual = capturesFor(Player.WHITE, Coord(3, 1), board)

        val expected = setOf(
            Coord(0, 0),
            Coord(1, 0),
            Coord(2, 0),
            Coord(2, 1)
        )

        assertEquals(expected, actual, "not greedy enough")
    }

    @Test
    fun notTooGreedy() {
        val pieces: Map<Coord, Player> = hashMapOf(
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

        val board = Board(pieces)

        val actual = capturesFor(Player.WHITE, Coord(3, 1), board)

        val expected: Set<Coord> = setOf()

        assertEquals(expected, actual, "too greedy")
    }

    @Test
    fun captureOtherSide() {
        val pieces: Map<Coord, Player> = hashMapOf(
            Pair(Coord(0, 0), Player.BLACK),
            Pair(Coord(1, 0), Player.BLACK),
            Pair(Coord(2, 0), Player.BLACK),
            Pair(Coord(0, 1), Player.WHITE),
            Pair(Coord(1, 1), Player.WHITE),
            Pair(Coord(2, 1), Player.BLACK),
            Pair(Coord(0, 2), Player.WHITE),
            Pair(Coord(1, 2), Player.WHITE),
            Pair(Coord(1, 3), Player.WHITE),
            Pair(Coord(1, 4), Player.WHITE),
            Pair(Coord(1, 5), Player.WHITE),
            Pair(Coord(2, 2), Player.WHITE),
            Pair(Coord(4, 3), Player.BLACK),
            Pair(Coord(3, 2), Player.BLACK),
            Pair(Coord(0, 3), Player.BLACK),
            Pair(Coord(2, 3), Player.BLACK),
            Pair(Coord(0, 4), Player.BLACK),
            Pair(Coord(2, 4), Player.BLACK),
            Pair(Coord(0, 5), Player.BLACK),
            Pair(Coord(2, 5), Player.BLACK)
        )

        val board = Board(pieces)

        val actual = capturesFor(Player.BLACK, Coord(1, 6), board)

        val expected: Set<Coord> = pieces.filterNot {
            it.value == Player.BLACK
                    || it.key == Coord(5, 1)
        }.map { it.key }.toSet()

        assertEquals(expected, actual, "erroneous aggression")
    }

    @Test
    fun cornerCaptureFullSizeBoard() {
        val pieces = hashMapOf(
            Pair(Coord(18, 18), Player.BLACK),
            Pair(Coord(18, 17), Player.WHITE)
        )

        val board = Board(pieces)

        val actual = capturesFor(Player.WHITE, Coord(17, 18), board)

        val expected: Set<Coord> = setOf(Coord(18, 18))

        assertEquals(expected, actual, "mind the corners")
    }

    @Test
    fun cornerLiberty() {
        val pieces = hashMapOf(
            Pair(Coord(18, 18), Player.BLACK),
            Pair(Coord(18, 17), Player.WHITE)
        )

        val board = Board(pieces)

        val actual = liberties(Coord(18, 18), board)

        val expected: Set<Coord> = setOf(Coord(17, 18))

        assertEquals(expected, actual, "cornered")
    }

    @Test
    fun connectedIncludesSelf() {
        val pieces = hashMapOf(
            Pair(Coord(18, 18), Player.BLACK),
            Pair(Coord(18, 17), Player.WHITE)
        )

        val board = Board(pieces)

        val actual = connected(Coord(18, 18), board)

        val expected: Set<Coord> = setOf(Coord(18, 18))

        assertEquals(expected, actual, "confused")
    }
}
