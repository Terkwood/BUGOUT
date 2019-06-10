# game judge

Uses [Kafka Streams](https://kafka.apache.org/22/documentation/streams/quickstart) to calculate game states.  Rejects invalid moves.  Declares the game completed when both parties pass

## streams example

A simple example of using Kafka streams 

https://github.com/gwenshap/kafka-streams-stockstats/blob/master/src/main/java/com/shapira/examples/streams/stockstats/StockStatsExample.java

## kafkacat crib notes

```sh
kafkacat -b 0.0.0.0:9092 -t bugout-make-move-cmd -P
{"gameId":"50b8d848-7c12-47fd-955f-c61c40d858af", "reqId":"50b8d848-7c12-47fd-955f-c61c40d858af", "player":"BLACK","coord":{"x":0,"y":0}}
```

```sh
kafkacat -b 0.0.0.0:9092 -t bugout-move-made-ev -C
```
