object Topics {
    const val CHOOSE_COLOR_PREF = "bugout-choose-color-pref-cmd"
    const val GAME_READY = "bugout-game-ready-ev"
    const val CLIENT_GAME_READY = "bugout-client-game-ready-ev"
    const val GAME_COLOR_PREF = "bugout-game-color-pref-ev"
    const val COLOR_PREFS_STORE = "bugout-color-prefs-store"
    const val COLORS_CHOSEN = "bugout-colors-chosen-ev"

    val all = arrayOf(CHOOSE_COLOR_PREF, GAME_READY, CLIENT_GAME_READY,
        GAME_COLOR_PREF, COLORS_CHOSEN)
}