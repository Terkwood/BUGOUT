/** A command to resign the game, sent by gateway when user clicks QUIT */
data class QuitGameCommand (val clientId: ClientId, val gameId: GameId)
