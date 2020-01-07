object Topics {
    const val GAME_READY = "bugout-game-ready-ev"
    const val FIND_PUBLIC_GAME = "bugout-find-public-game-cmd"
    const val CREATE_GAME = "bugout-create-game-cmd"
    const val JOIN_PRIVATE_GAME = "bugout-join-private-game-cmd"

    const val QUIT_GAME = "bugout-quit-game-cmd"

    val all = arrayOf(GAME_READY, QUIT_GAME)
}
