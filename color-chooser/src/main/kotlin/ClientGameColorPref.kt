import serdes.jsonMapper

data class ClientGameColorPref(val clientId: ClientId,
                               val gameId: GameId,
                               val colorPref: ColorPref)