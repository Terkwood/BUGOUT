#!/bin/bash

# You must provide an initial game state, or you'll blow up Judge (https://github.com/Terkwood/BUGOUT/issues/22)
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"board":{"pieces":{},"size":19},"captures":{"black":0,"white":0},"turn":1,"playerUp":"BLACK"}' | kafkacat -b kafka:9092 -t bugout-game-states -K: -P
# Allow message above to trigger cold start in gamestates-aggregator
sleep 8
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"0000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":0,"y":0}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
# Allow message above to trigger cold start in judge
sleep 8
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"1000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":0}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
sleep 1
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"2000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":10,"y":11}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
sleep 1
echo 'a0b8d848-7c12-47fd-955f-c61c40d858af:{"gameId":"a0b8d848-7c12-47fd-955f-c61c40d858af","reqId":"3000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":0,"y":1}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P

# run another game
echo 'deadbeef-7c12-47fd-955f-c61c40d858af:{"board":{"pieces":{},"size":19},"captures":{"black":0,"white":0},"turn":1,"playerUp":"BLACK"}' | kafkacat -b kafka:9092 -t bugout-game-states -K: -P
sleep 1
echo 'deadbeef-7c12-47fd-955f-c61c40d858af:{"gameId":"deadbeef-7c12-47fd-955f-c61c40d858af","reqId":"0000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":0,"y":0}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
sleep 1
echo 'deadbeef-7c12-47fd-955f-c61c40d858af:{"gameId":"deadbeef-7c12-47fd-955f-c61c40d858af","reqId":"1000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":1,"y":0}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
sleep 1
echo 'deadbeef-7c12-47fd-955f-c61c40d858af:{"gameId":"deadbeef-7c12-47fd-955f-c61c40d858af","reqId":"2000b0e5-a943-491a-938a-19a35677a501", "player":"BLACK","coord":{"x":10,"y":11}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
sleep 1
echo 'deadbeef-7c12-47fd-955f-c61c40d858af:{"gameId":"deadbeef-7c12-47fd-955f-c61c40d858af","reqId":"3000b0e5-a943-491a-938a-19a35677a501", "player":"WHITE","coord":{"x":0,"y":1}}'| kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P

kafkacat -b kafka:9092 -t bugout-move-made-ev -C -K: 
