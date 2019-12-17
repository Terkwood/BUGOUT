import org.apache.kafka.streams.kstream.KStream
import org.apache.kafka.streams.kstream.Predicate

fun <K, V> KStream<K, V>.kbranch(vararg predicates: (K, V) -> Boolean):
        Array<KStream<K, V>> {
    val arguments: List<Predicate<K, V>> =
        predicates.map { Predicate { key: K, value: V -> it(key, value) } }
    return this.branch(*arguments.toTypedArray())
}
