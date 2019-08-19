import serdes.jsonMapper

/** Emitted downstream of GameReady.  Client ID is
 * topic key.
 */
data class ClientGameReady (
        val clientId: ClientId,
        val gameId: GameId
){
        fun asByteArray(): ByteArray = jsonMapper.writeValueAsBytes(this)
}
