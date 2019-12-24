import serdes.jsonMapper

class AggregatedPrefs {
    var prefs: ArrayList <SessionGameColorPref> = arrayListOf()
    fun add(p: SessionGameColorPref) = prefs.add(p)
    fun asByteArray(): ByteArray = jsonMapper.writeValueAsBytes(this)
}