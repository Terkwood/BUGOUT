/** Return all open spaces connected to the target piece */
fun liberties(target: Coord, board: Board): Set<Coord> = TODO()

/* Return neighbors on (up to) four sides of the target */
fun neighbors(target: Coord, board: Board): Set<Pair<Coord, Player>> =
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
    }.map {
        val c = board.pieces[it]
        if (c == null) null else Pair(it, c)
    }.filterNotNull().toSet()


fun deadFrom(target: Coord, placement: Coord, board: Board):
        Boolean {
    val targetLiberties = liberties(target, board)
    return targetLiberties.size == 1 && targetLiberties.take(1)[0] ==
            placement
}

/** Return all pieces of the same color, connected to the target.  Includes
 * the target itself.
 */
fun connected(target: Coord, board: Board): Set<Coord> = TODO()

/** Returns a set of all coordinates captured by `player` placing a piece at
 * `placement` */
fun capturesFor(player: Player, placement: Coord, board: Board): Set<Coord> {
    val enemyNeighbors =
        neighbors(placement, board).filter {
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
