import serdes.jsonMapper

class AggregatedPrefs {
    var prefs: ArrayList <ClientGameColorPref> = arrayListOf()
    fun add(p: ClientGameColorPref) = prefs.add(p)
    fun asByteArray(): ByteArray = jsonMapper.writeValueAsBytes(this)
}