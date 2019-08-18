#!/bin/bash

# key is clientID
echo 'ddeaa10a-c3d3-475f-b33d-6addee2857bc:{"gameId":"0050a3fc-aaaa-4c9c-95f1-5c67f5e979aa", "clientId": "ddeaa10a-c3d3-475f-b33d-6addee2857bc", "visibility": "Private"}' | kafkacat -b kafka:9092 -t bugout-create-game-cmd -K: -P
