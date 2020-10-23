object Topics {
    const val MOVE_ACCEPTED_EV = "bugout-move-accepted-ev"
    const val MOVE_MADE_EV = "bugout-move-made-ev"
    const val GAME_STATES_STORE_NAME = "bugout-game-states-store"
    const val GAME_STATES_CHANGELOG = "bugout-game-states"
    const val GAME_READY = "bugout-game-ready-ev"

    val all = arrayOf(MOVE_ACCEPTED_EV,
        MOVE_MADE_EV,
        GAME_STATES_CHANGELOG,
        GAME_READY)
}