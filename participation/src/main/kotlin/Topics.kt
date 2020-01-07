object Topics {
    const val GAME_READY = "bugout-game-ready-ev"
    const val FIND_PUBLIC_GAME = "bugout-find-public-game-cmd"
    const val CREATE_GAME = "bugout-create-game-cmd"
    const val JOIN_PRIVATE_GAME = "bugout-join-private-game-cmd"

    const val GAME_PARTICIPATION = "bugout-game-participation-ev"

    const val QUIT_GAME = "bugout-quit-game-cmd"

    const val PUBLIC_GAME_AGGREGATE_STORE = "bugout-public-game-aggregate-store"

    val all = arrayOf(GAME_READY, FIND_PUBLIC_GAME, CREATE_GAME, JOIN_PRIVATE_GAME, QUIT_GAME, PUBLIC_GAME_AGGREGATE_STORE)
}
