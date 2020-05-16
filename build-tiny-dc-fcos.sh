#!/bin/bash

/opt/bin/docker-compose -f dc-tiny.yml build gateway
/opt/bin/docker-compose -f dc-tiny.yml build micro-judge
/opt/bin/docker-compose -f dc-tiny.yml build micro-changelog
/opt/bin/docker-compose -f dc-tiny.yml build micro-game-lobby
/opt/bin/docker-compose -f dc-tiny.yml build bugle
/opt/bin/docker-compose -f dc-tiny.yml build botlink
