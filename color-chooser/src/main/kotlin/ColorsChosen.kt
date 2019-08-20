data class ColorsChosen(val gameId: GameId, val black: ClientId, val white: ClientId) {

    companion object {
        fun resolve(one: ClientGameColorPref, another: ClientGameColorPref): ColorsChosen {
            val x = Pair(one, another)
            return when {
                isAny(x.first.colorPref) -> TODO("any first")
                isAny(x.second.colorPref) -> TODO("any second")
                // no conflict
                x.first.colorPref == ColorPref.Black && x.first != x.second ->
                    ColorsChosen(
                        gameId = x.first.gameId,
                        black = x.first.clientId,
                        white = x.second.clientId
                    )
                // no conflict
                x.first.colorPref == ColorPref.White && x.first != x.second ->
                    ColorsChosen(
                        gameId = x.first.gameId,
                        white = x.first.clientId,
                        black = x.second.clientId
                    )
                else -> TODO("conflict")
            }
        }

        private fun random(): Color = TODO()

        private fun force(cp: ColorPref): Color =
            when (cp) {
                ColorPref.Any -> random()
                ColorPref.Black -> Color.Black
                ColorPref.White -> Color.White
            }


        private fun other(c: Color) = when (c) {
            Color.Black -> Color.White
            Color.White -> Color.Black
        }

    }
}

enum class Color { Black, White }