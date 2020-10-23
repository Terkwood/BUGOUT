#!/bin/bash

docker cp src bugout_sync_1:/home/gradle/. \
    && docker exec -it bugout_sync_1 bash
