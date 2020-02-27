/** Return all open spaces connected to the target piece's formation */
fun liberties(target: Coord, board: Board): Set<Coord> =
    connected(target, board).flatMap { neighborSpaces(it, board) }.toSet()


fun neighbors(target: Coord, board: Board): Set<Pair<Coord, Player?>> =
    listOf(
        Pair(-1, 0),
        Pair(1, 0),
        Pair(0, -1),
        Pair(0, 1)
    ).asSequence().map { (x, y) ->
        Coord(
            x + target.x,
            y + target.y
        )
    }.filterNot {
        it.x < 0
                || it.x >= board.size
                || it.y < 0
                || it.y >= board.size
    }.map { Pair(it, board.pieces[it]) }.toSet()

/** Return neighboring empty spaces */
fun neighborSpaces(target: Coord, board: Board): Set<Coord> =
    neighbors(target, board).mapNotNull {
        if (it.second != null) null else it.first
    }.toSet()

/* Return neighborPieces on all sides of the target */
fun neighborPieces(target: Coord, board: Board): Set<Pair<Coord, Player>> =
    neighbors(target, board).mapNotNull {
        val p = it.second
        if (p == null) null else Pair(it.first, p)
    }.toSet()


fun deadFrom(target: Coord, placement: Coord, board: Board):
        Boolean =
    liberties(target, board) == setOf(placement)

/** Return all pieces of the same color, connected to the target.  Includes
 * the target itself.
 */
fun connected(target: Coord, board: Board): Set<Coord> {
    val player = board.pieces[target] ?: return setOf()

    tailrec fun halp(targets: Set<Coord>, acc: Set<Coord>): Set<Coord> {
        val sameColorPieces = targets.mapNotNull {
            val found = board.pieces[it]
            if (found == null) null else {
                Pair(it, found)
            }
        }.filter { it.second == player }.map { it.first }

        val sameColorNeighbors: Set<Coord> =
            sameColorPieces
                .flatMap {
                    neighborPieces(
                        it,
                        board
                    )
                }.filter {
                    it.second ==
                            player
                }.map {
                    it
                        .first
                }.toSet()

        return if (acc.containsAll(sameColorNeighbors))
            acc
        else {
            val nextAcc = acc.union(sameColorNeighbors)
            halp(sameColorNeighbors, nextAcc)
        }
    }

    return halp(setOf(target), setOf(target))
}

/** Returns a set of all coordinates captured by `player` placing a piece at
 * `placement` */
fun capturesFor(player: Player, placement: Coord, board: Board): Set<Coord> {
    val enemyNeighbors =
        neighborPieces(placement, board).filter {
            it.second != player
        }.map { Pair(it.first, it.second) }
    val someDead = enemyNeighbors.map { (target, _) ->
        if (deadFrom(target, placement, board)) {
            connected(target, board)
        } else
            HashSet()
    }
    return someDead.flatten().toSet()
}

