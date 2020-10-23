#!/bin/bash

# we assume there's already a lobby entry in the system,
# as initialized by the startup container's feed

echo 'ALL:{"games":[]}' | kafkacat -b kafka:9092 -t bugout-game-lobby -K: -P

echo '22ade809-07c0-452f-bbce-b63e326006bd:{"clientId":"22ade809-07c0-452f-bbce-b63e326006bd"}' | kafkacat -b kafka:9092 -t bugout-find-public-game-cmd -K: -P

sleep 1

echo '332dd50b-4660-4531-a3fe-971d4773b0af:{"clientId":"332dd50b-4660-4531-a3fe-971d4773b0af"}' | kafkacat -b kafka:9092 -t bugout-find-public-game-cmd -K: -P

kafkacat -b kafka:9092 -t bugout-game-ready-ev -C -K: 
