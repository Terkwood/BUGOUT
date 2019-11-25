object Topics {
    const val MOVE_ACCEPTED_EV_TOPIC = "bugout-move-accepted-ev"
    const val MOVE_MADE_EV_TOPIC = "bugout-move-made-ev"
    const val GAME_STATES_STORE_NAME = "bugout-game-states-store"
    const val GAME_STATES_CHANGELOG_TOPIC = "bugout-game-states"

    val all = arrayOf(MOVE_ACCEPTED_EV_TOPIC, MOVE_MADE_EV_TOPIC, GAME_STATES_CHANGELOG_TOPIC)
}