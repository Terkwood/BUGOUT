object Topics {
    const val PROVIDE_HISTORY_CMD = "bugout-provide-history-cmd"
    const val HISTORY_PROVIDED_EV = "bugout-history-provided-ev"

    const val MAKE_MOVE_CMD = "bugout-make-move-cmd"
    const val MOVE_MADE_EV = "bugout-move-made-ev"
    const val REQ_SYNC_CMD = "bugout-req-sync-cmd"
    const val SYNC_REPLY_EV = "bugout-sync-reply-ev"

    val all = arrayOf(PROVIDE_HISTORY_CMD, HISTORY_PROVIDED_EV,
            MAKE_MOVE_CMD, MOVE_MADE_EV, REQ_SYNC_CMD, SYNC_REPLY_EV)
}
