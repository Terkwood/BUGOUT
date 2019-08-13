#!/bin/bash

echo 'ALL:{"games":[]}' | kafkacat -b kafka:9092 -t bugout-game-lobby -K: -P