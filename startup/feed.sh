#!/bin/bash

./wait-for-it.sh kafka:9092 -s -- sleep 16

# make sure the game lobby has an empty entry
echo 'ALL:{"games":[]}' | kafkacat -b kafka:9092 -t bugout-game-lobby -K: -P

echo "🚪 STARTUP COMPLETE - EXITING!"
