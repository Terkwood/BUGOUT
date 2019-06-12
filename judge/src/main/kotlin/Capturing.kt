/** Return all open spaces connected to the target piece */
fun liberties(target: Coord, board: Board): Set<Coord> = TODO()

/* Return neighbors on (up to) four sides of the target */
fun neighbors(target: Coord, board: Board): HashSet<Pair<Player, Coord>> =
    TODO()

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
        neighbors(placement, board).filter { it.first != player }
    val someDead = enemyNeighbors.map { (_, target) ->
        if (deadFrom(target, placement, board)) {
            connected(target, board)
        } else
            HashSet()
    }
    return someDead.flatten().toSet()
}
