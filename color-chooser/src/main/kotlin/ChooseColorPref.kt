/** A client's chosen color preference.  Keyed by client ID */
data class ChooseColorPref(
        val clientId: ClientId,
        val colorPref: ColorPref
)