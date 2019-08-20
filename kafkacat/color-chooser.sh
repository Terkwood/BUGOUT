#!/bin/bash

# need a gameready event to have been emitted by game lobby
# keyed by game ID
echo 'ffff0000-bbab-bbbb-5434-b63e326006bd:{"gameId":"ffff0000-bbab-bbbb-5434-b63e326006bd", "clients": {"first":"eeeeeeee-fefe-1234-ffff-d5f736841a5f","second":"bbbbbbbb-8787-6a6a-5432-bce65874eed2"}, "eventId":"3f1eac1e-ed30-4fdf-811a-eaf9eba01cc5"}' | kafkacat -b kafka:9092 -t bugout-game-ready-ev -K: -P


# each player chooses a color pref
# keyed by client ID
echo 'eeeeeeee-fefe-1234-ffff-d5f736841a5f:{"clientId":"eeeeeeee-fefe-1234-ffff-d5f736841a5f", "colorPref":"Black"}' | kafkacat -b kafka:9092 -t bugout-choose-color-pref-cmd -K: -P
echo 'bbbbbbbb-8787-6a6a-5432-bce65874eed2:{"clientId":"bbbbbbbb-8787-6a6a-5432-bce65874eed2", "colorPref":"Black"}' | kafkacat -b kafka:9092 -t bugout-choose-color-pref-cmd -K: -P


# wait for the magic to happen
# keyed by game ID
kafkacat -b kafka:9092 -t bugout-colors-chosen-ev -C -K: 
