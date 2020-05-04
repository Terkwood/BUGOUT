data class ReqSyncCmd(
    val sessionId: SessionId,
    val reqId: ReqId,
    val gameId: GameId,
    val playerUp: Player,
    val turn: Int,
    val lastMove: Move?
)

data class SyncReplyEv(
    val sessionId: SessionId,
    val replyTo: ReqId,
    val gameId: GameId,
    val playerUp: Player,
    val turn: Int,
    val moves: List<Move> = ArrayList()
)
