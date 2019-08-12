import serdes.jsonMapper

data class OpenGame(val gameId: GameId, val visibility: Visibility) {
    fun asByteArray(): ByteArray {
        return jsonMapper.writeValueAsBytes(this)
    }
}
