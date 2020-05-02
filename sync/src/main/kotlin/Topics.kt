object Topics {
    const val GAME_STATES_CHANGELOG = "bugout-game-states-changelog"
    const val MAKE_MOVE_CMD = "bugout-make-move-cmd"
    const val REQ_SYNC_CMD = "bugout-req-sync-cmd"
    const val SYNC_REPLY_EV = "bugout-sync-reply-ev"

    const val LOCAL_GAME_STATES_STORE = "bugout-sync-game-states"

    val all = arrayOf(GAME_STATES_CHANGELOG, MAKE_MOVE_CMD, REQ_SYNC_CMD, SYNC_REPLY_EV, LOCAL_GAME_STATES_STORE)
}
