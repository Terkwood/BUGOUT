import serdes.jsonMapper

/** A client's chosen color preference.  Keyed by client ID */
data class ChooseColorPref(
        val clientId: ClientId,
        val colorPref: ColorPref
){
        fun asByteArray(): ByteArray = jsonMapper.writeValueAsBytes(this)
}
