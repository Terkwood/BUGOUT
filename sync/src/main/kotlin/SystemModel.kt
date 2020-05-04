/** This command requests that a move be judged for
 * validity and communicated to all other participants.
 */
data class MakeMoveCmd(
    val gameId: GameId,
    val reqId: ReqId,
    val player: Player,
    val coord: Coord?
)

/**
 * An event signalling the acceptance of a move.
 * This is emitted by changelog service.
 */
data class MoveMadeEv(
    val gameId: GameId,
    val replyTo: ReqId,
    val eventId: EventId,
    val player: Player,
    val coord: Coord?,
    val captured: List<Coord> = ArrayList()
)
