import kotlin.random.Random

data class ColorsChosen(val gameId: GameId, val black: ClientId, val white: ClientId) {

    companion object {
        fun resolve(first: ClientGameColorPref, second: ClientGameColorPref): ColorsChosen {

            return when {
                isAny(first.colorPref) ->
                    when (force(second.colorPref)) {
                        Color.Black -> whiteFirst(first, second)
                        Color.White -> blackFirst(first, second)
                    }
                isAny(second.colorPref) ->
                    when (force(first.colorPref)) {
                        Color.Black -> blackFirst(first, second)
                        Color.White -> whiteFirst(first, second)
                    }
                // no conflict
                first.colorPref == ColorPref.Black && first.colorPref != second.colorPref ->
                    blackFirst(first, second)
                // no conflict
                first.colorPref == ColorPref.White && first != second ->
                    whiteFirst(first, second)
                // both sides picked the same color
                else -> {
                    return when (random()) {
                        Color.Black -> blackFirst(first, second)
                        Color.White -> whiteFirst(first, second)
                    }
                }
            }
        }

        private fun blackFirst(first: ClientGameColorPref, second: ClientGameColorPref) =
            ColorsChosen(
                gameId = first.gameId,
                black = first.clientId,
                white = second.clientId
            )

        private fun whiteFirst(first: ClientGameColorPref, second: ClientGameColorPref) =
            ColorsChosen(
                gameId = first.gameId,
                black = first.clientId,
                white = second.clientId
            )

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