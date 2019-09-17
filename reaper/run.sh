#!/bin/bash

# Give it some extra time to let the rest of the system start
# up and process messages
./wait-for-it.sh kafka:9092 -s -- sleep 60
reaper
