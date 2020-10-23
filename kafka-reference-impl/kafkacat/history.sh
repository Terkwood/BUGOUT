#!/bin/bash

echo '507e5048-7f0a-4fb0-9270-689010fa2461:{"gameId":"507e5048-7f0a-4fb0-9270-689010fa2461", "reqId": "1b51f374-c970-4008-ad42-6c5e2206a127"}' | kafkacat -b kafka:9092 -t bugout-provide-history-cmd -K: -P
