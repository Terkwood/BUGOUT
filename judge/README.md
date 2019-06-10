```sh
kafkacat -b 0.0.0.0:9092 -t bugout-make-move-cmd -P
```

```sh
kafkacat -b 0.0.0.0:9092 -t bugout-move-made-ev -C
```

```json
{"gameId":"50b8d848-7c12-47fd-955f-c61c40d858af", "reqId":"50b8d848-7c12-47fd-955f-c61c40d858af", "player":"BLACK","coord":{"x":0,"y":0}}
```