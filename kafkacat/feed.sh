#!/bin/bash

echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"board":{"pieces":{},"size":19},"captures":{"black":0,"white":0},"turn":2,"playerUp":"WHITE"}' | kafkacat -b kafka:9092 -t bugout-game-states  -K: -P
sleep 5
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"0000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":0,"y":0}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
sleep 5
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"1000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":0}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
sleep 5
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"2000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":0,"y":1}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
sleep 5
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"3000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":1}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P

kafkacat -b kafka:9092 -t bugout-move-made-ev -C -K: 

echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"3000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":1}}' | kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
