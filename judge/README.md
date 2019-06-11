# game judge

Uses [Kafka Streams](https://kafka.apache.org/22/documentation/streams/quickstart) to calculate game states.  Rejects invalid moves.  Declares the game completed when both parties pass

## streams example

Here's [a reference example for using Kafka streams](https://github.com/gwenshap/kafka-streams-stockstats/blob/master/src/main/java/com/shapira/examples/streams/stockstats/StockStatsExample.java).

There's [another helpful example in this project](https://github.com/adrien-ben/kstream-aggregation-example/blob/master/src/main/kotlin/com/boulanger/poc/salesaggregation/Configuration.kt).

There's [yet another example here](https://github.com/stream1984/kafka-stream-examples/blob/master/src/main/kotlin/cn/leapcloud/watchout/WatchHTTPStatus.kt).

## topic testing

You need to run these commands from inside the `kafka` docker
container in order for their hostname to work correctly.

Produce an event to the Make Move Command topic:

```sh
kafka-console-producer.sh --broker-list kafka:9092 --topic bugout-make-move-cmd
>{"gameId":"50b8d848-7c12-47fd-955f-c61c40d858af","reqId":"50b8d848-7c12-47fd-955f-c61c40d858af", "player":"BLACK","coord":{"x":0,"y":0}}
```

Consume an event from the Move Made Event topic:

```sh
kafka-console-consumer.sh --bootstrap-server kafka:9092 --topic bugout-move-made-ev --from-beginning
```

## kafkacat crib notes

Note that the IP address of 0.0.0.0 is used as an example of connecting from a host such as a Mac.

```sh
kafkacat -b 0.0.0.0:9092 -t bugout-make-move-cmd -P
{"gameId":"50b8d848-7c12-47fd-955f-c61c40d858af", "reqId":"50b8d848-7c12-47fd-955f-c61c40d858af", "player":"BLACK","coord":{"x":0,"y":0}}
```

```sh
kafkacat -b 0.0.0.0:9092 -t bugout-move-made-ev -C
```
