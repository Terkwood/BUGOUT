#!/bin/bash

# need a gameready event to have been emitted by game lobby
# keyed by game ID
echo 'aaade809-1c1c-452f-bbce-b63e326006bd:{"gameId":"aaade809-1c1c-452f-bbce-b63e326006bd", "clients": {"first":"c24073ac-fefe-4854-a380-d5f736841a5f","second":"01c3803c-3849-6a6a-940d-bce65874eed2"}, "eventId":"3f1eac1e-ed30-4fdf-811a-eaf9eba01cc5"}' | kafkacat -b kafka:9092 -t bugout-game-ready-ev -K: -P


# each player chooses a color pref
# keyed by client ID
echo 'c24073ac-fefe-4854-a380-d5f736841a5f:{"clientId":"c24073ac-fefe-4854-a380-d5f736841a5f", "colorPref":"Black"}' | kafkacat -b kafka:9092 -t bugout-choose-color-pref-cmd -K: -P
echo '01c3803c-3849-6a6a-940d-bce65874eed2:{"clientId":"01c3803c-3849-6a6a-940d-bce65874eed2", "colorPref":"Black"}' | kafkacat -b kafka:9092 -t bugout-choose-color-pref-cmd -K: -P


# wait for the magic to happen
# keyed by game ID
kafkacat -b kafka:9092 -t bugout-colors-chosen-ev -C -K: 
