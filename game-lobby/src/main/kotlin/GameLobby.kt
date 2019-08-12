import java.util.*

fun main() {
    TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
    GameLobby("kafka:9092").process()
}

class GameLobby(val brokers: String) {
    fun process() {
        print("👺")
    }
}