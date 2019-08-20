import kotlin.random.Random

data class ColorsChosen(val gameId: GameId, val black: ClientId, val white: ClientId) {

    companion object {
        fun resolve(first: ClientGameColorPref, second: ClientGameColorPref): ColorsChosen {
            val noConflict: Boolean by lazy { first.colorPref != second.colorPref }
            val bf: ColorsChosen by lazy { blackFirst(first, second) }
            val wf: ColorsChosen by lazy { whiteFirst(first, second) }
            return when {
                isAny(first.colorPref) ->
                    when (force(second.colorPref)) {
                        Color.Black -> wf
                        Color.White -> bf
                    }
                isAny(second.colorPref) ->
                    when (force(first.colorPref)) {
                        Color.Black -> bf
                        Color.White -> wf
                    }
                first.colorPref == ColorPref.Black && noConflict -> 
                    bf
                first.colorPref == ColorPref.White && noConflict ->
                    wf
                // both sides picked the same color
                else ->
                    when (random()) {
                        Color.Black -> blackFirst(first, second)
                        Color.White -> whiteFirst(first, second)
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
                black = second.clientId,
                white = first.clientId
            )

        private fun random(): Color = when (Random.nextBoolean()) {
            false -> Color.Black
            true -> Color.White
        }

        private fun force(cp: ColorPref): Color =
            when (cp) {
                ColorPref.Any -> random()
                ColorPref.Black -> Color.Black
                ColorPref.White -> Color.White
            }


    }
}

enum class Color { Black, White }