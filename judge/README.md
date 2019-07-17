# game judge

Uses [Kafka Streams](https://kafka.apache.org/22/documentation/streams/quickstart) to calculate game states. [Responsible for rejecting invalid moves](https://github.com/Terkwood/BUGOUT/issues/20).

## kafka streams topology

![game judge topology](topology.jpg)

## stream examples

Here's [a reference example for using Kafka streams](https://github.com/gwenshap/kafka-streams-stockstats/blob/master/src/main/java/com/shapira/examples/streams/stockstats/StockStatsExample.java).

There's [another helpful example in this project](https://github.com/adrien-ben/kstream-aggregation-example/blob/master/src/main/kotlin/com/boulanger/poc/salesaggregation/Configuration.kt).

There's [yet another example here](https://github.com/stream1984/kafka-stream-examples/blob/master/src/main/kotlin/cn/leapcloud/watchout/WatchHTTPStatus.kt).

More [inspiration here](https://blog.softwaremill.com/event-sourcing-using-kafka-53dfd72ad45d).

And [some inspiration here](https://medium.com/@abhishek1987/kafka-streams-interactive-queries-9a05ff92d75a).

## pushing data around by hand

You need to run these commands from inside the `kafka` docker
container in order for their hostname to work correctly.

Produce an event to the Make Move Command topic:

```sh
kafka-console-producer.sh --broker-list kafka:9092 --topic bugout-make-move-cmd
```

```json
{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"0000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":0,"y":0}}
{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"1000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":0}}
{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"2000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":0,"y":1}}
{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"3000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":1}}

```

```json
{"gameId":"deadbeef-aaaa-aaaa-955f-c61c40d858af","reqId":"0000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":0,"y":0}}
{"gameId":"deadbeef-aaaa-aaaa-955f-c61c40d858af","reqId":"1000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":0}}
{"gameId":"deadbeef-aaaa-aaaa-955f-c61c40d858af","reqId":"2000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":0,"y":1}}
{"gameId":"deadbeef-aaaa-aaaa-955f-c61c40d858af","reqId":"3000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":1}}
```

Consume an event from the Move Made Event topic:

```sh
kafka-console-consumer.sh --bootstrap-server kafka:9092 --topic bugout-move-made-ev --from-beginning
```

Or produce to Move Made Event topic:

```sh
kafka-console-producer.sh --broker-list kafka:9092 --topic bugout-move-made-ev
```

Examples of moves made

```json
{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","replyTo":"0000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","captured": [],"coord":{"x":0,"y":0}}
{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","replyTo":"1000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","captured": [],"coord":{"x":1,"y":0}}
```

Consume an event from the state changelog:

```sh
kafka-console-consumer.sh --bootstrap-server kafka:9092 --topic bugout-game-states --from-beginning
```

Or even push to the state changelog to bootstrap your joins:

```sh
kafka-console-producer.sh --broker-list kafka:9092 --topic bugout-game-states
```

Changelog records such as:

```json
{"board":{"pieces":{"0,0":"BLACK"},"size":19},"captures":{"black":0,"white":0},"turn":2,"playerUp":"WHITE"}
```

## kafkacat notes

Note that the IP address of 0.0.0.0 is used as an example of connecting from a host such as a Mac.

```sh
kafkacat -b 0.0.0.0:9092 -t bugout-make-move-cmd -P
{"gameId":"50b8d848-7c12-47fd-955f-c61c40d858af", "reqId":"50b8d848-7c12-47fd-955f-c61c40d858af", "player":"BLACK","coord":{"x":0,"y":0}}
```

```sh
kafkacat -b 0.0.0.0:9092 -t bugout-move-made-ev -C
```

## Docker + Kafka notes

[Read wurstmeister's connectivity wiki](https://github.com/wurstmeister/kafka-docker/wiki/Connectivity) and [read this article](https://www.kaaproject.org/kafka-docker).
