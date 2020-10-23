#!/bin/bash

docker cp src bugout_participation_1:/home/gradle/. \
    && docker exec -it bugout_participation_1 bash
