#!/bin/bash

# key is clientID
echo 'bbeaa10a-c3d3-475f-b33d-6addee2857bc:{"gameId":"ad50a3fc-aaaa-4c9c-95f1-5c67f5e979aa", "clientId": "bbeaa10a-c3d3-475f-b33d-6addee2857bc", "visibility": "Private"}' | kafkacat -b kafka:9092 -t bugout-create-game-cmd -K: -P
