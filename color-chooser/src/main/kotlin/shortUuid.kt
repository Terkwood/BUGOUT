import java.util.*

const val SIZE = 8
fun UUID.short(): String = this.toString().take(SIZE)
