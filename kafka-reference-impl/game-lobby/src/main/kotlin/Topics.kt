object Topics {
    const val FIND_PUBLIC_GAME = "bugout-find-public-game-cmd"
    const val CREATE_GAME = "bugout-create-game-cmd"
    const val WAIT_FOR_OPPONENT = "bugout-wait-for-opponent-ev"
    const val GAME_READY = "bugout-game-ready-ev"
    const val PRIVATE_GAME_REJECTED = "bugout-private-game-rejected-ev"
    const val JOIN_PRIVATE_GAME = "bugout-join-private-game-cmd"

    const val GAME_LOBBY_COMMANDS = "bugout-game-lobby-commands"
    const val GAME_LOBBY_CHANGELOG = "bugout-game-lobby"

    const val GAME_STATES_CHANGELOG = "bugout-game-states"
    
    const val SESSION_DISCONNECTED = "bugout-session-disconnected-ev"

    // kv stores
    const val GAME_LOBBY_STORE_LOCAL = "bugout-game-lobby-store-local"
    const val GAME_LOBBY_STORE_GLOBAL = "bugout-game-lobby-store-global"


    val all = arrayOf(FIND_PUBLIC_GAME, CREATE_GAME, WAIT_FOR_OPPONENT,
        GAME_READY, PRIVATE_GAME_REJECTED, JOIN_PRIVATE_GAME,
        GAME_LOBBY_COMMANDS, GAME_LOBBY_CHANGELOG, GAME_STATES_CHANGELOG,
        SESSION_DISCONNECTED)
}
