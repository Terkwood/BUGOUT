data class MakeMoveCmd(
    val gameId: GameId,
    val reqId: ReqId,
    val player: Player,
    val coord: Coord?
)
