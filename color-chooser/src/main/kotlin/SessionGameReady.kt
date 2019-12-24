import serdes.jsonMapper

/** Emitted downstream of GameReady.  Session ID is
 * topic key.
 */
data class SessionGameReady (
        val sessionId: SessionId,
        val gameId: GameId
){
        fun asByteArray(): ByteArray = jsonMapper.writeValueAsBytes(this)
}
