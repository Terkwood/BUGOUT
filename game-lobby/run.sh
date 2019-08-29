#!/bin/bash

# increasing the sleep vs other services since
# this seems to hang  -- 2019-08-29
./wait-for-it.sh kafka:9092 -s -- sleep 20
java -jar bugout.game-lobby.jar
