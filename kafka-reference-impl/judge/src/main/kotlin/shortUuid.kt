import java.util.*

fun UUID.short(): String = this.toString().take(8)
