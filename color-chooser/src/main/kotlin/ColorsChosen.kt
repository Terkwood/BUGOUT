data class ColorsChosen(val gameId: GameId, val black: ClientId, val white: ClientId) {

    companion object {
        fun resolve(one: ClientGameColorPref, another: ClientGameColorPref): ColorsChosen {
            val x = Pair(one, another)
            return when {
                isAny(x.first.colorPref) -> TODO("any first")
                isAny(x.second.colorPref) -> TODO("any second")
                x.first == x.second -> TODO("conflict")
                // no conflict below
                x.first.colorPref == ColorPref.Black ->
                    ColorsChosen(
                        gameId = x.first.gameId,
                        black = x.first.clientId,
                        white = x.second.clientId)
                x.first.colorPref == ColorPref.White ->
                    ColorsChosen(
                        gameId = x.first.gameId,
                        white = x.first.clientId,
                        black = x.second.clientId)
                else -> TODO("else")
            }
        }
    }
}
