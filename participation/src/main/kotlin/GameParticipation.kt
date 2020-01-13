data class GameParticipation(
    val gameId: GameId,
    val clients: Pair<ClientId, ClientId>,
    val participation: Participation
)
