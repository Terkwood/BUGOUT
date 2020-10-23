object Topics {
    const val GAME_STATES_STORE_NAME =
        "bugout-game-states-store-history-provider"
    const val GAME_STATES_CHANGELOG_TOPIC = "bugout-game-states"
    const val PROVIDE_HISTORY_TOPIC = "bugout-provide-history-cmd"
    const val HISTORY_PROVIDED_TOPIC = "bugout-history-provided-ev"

    val all = arrayOf(GAME_STATES_CHANGELOG_TOPIC, PROVIDE_HISTORY_TOPIC, HISTORY_PROVIDED_TOPIC)
}