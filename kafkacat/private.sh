#!/bin/bash

# key is clientID
echo '111aa10a-aaaa-475f-b33d-6addee2857aa:{"gameId":"0050a3fc-2222-3333-95f1-5c67f5e979bb", "clientId": "111aa10a-aaaa-475f-b33d-6addee2857aa", "visibility": "Private"}' | kafkacat -b kafka:9092 -t bugout-create-game-cmd -K: -P
