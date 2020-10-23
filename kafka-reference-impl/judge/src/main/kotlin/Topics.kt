const val MAKE_MOVE_CMD_TOPIC = "bugout-make-move-cmd"
const val MAKE_MOVE_CMD_DEDUP_TOPIC = "bugout-make-move-cmd-dedup"
const val MOVE_ACCEPTED_EV_TOPIC = "bugout-move-accepted-ev"
const val MOVE_REJECTED_EV_TOPIC = "bugout-move-rejected-ev"
const val GAME_STATES_CHANGELOG_TOPIC = "bugout-game-states"
const val GAME_STATES_STORE = "bugout-game-states-store-judge"

const val MAKE_MOVE_DEDUP_STORE = "bugout-judge-make-move-cmd-dedup-store"

val ALL_TOPICS = arrayOf(GAME_STATES_CHANGELOG_TOPIC, MAKE_MOVE_CMD_TOPIC, MOVE_ACCEPTED_EV_TOPIC, MOVE_REJECTED_EV_TOPIC,MAKE_MOVE_CMD_DEDUP_TOPIC)
