#!/bin/bash

kafkacat -b kafka:9092 -t bugout-game-states -T -P -l msgs-game-states -K:
kafkacat -b kafka:9092 -t bugout-make-move-cmd -T -P -l msgs-make-move-cmd -K:
echo "Done producing startup messages"
kafkacat -b kafka:9092 -t bugout-move-made-ev -C -K: 
tail -f /dev/null
kafkacat -b kafka:9092 -t bugout-make-move-cmd -K: -P
