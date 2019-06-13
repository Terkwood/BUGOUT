fun main() {
    Aggregator("kafka:9092").process()
}

class Aggregator(private val brokers: String) {
    fun process() {}
}
