#!/bin/bash

export COMPOSE_DOCKER_CLI_BUILD=1 
export DOCKER_BUILDKIT=1
/usr/bin/docker-compose build gateway
/usr/bin/docker-compose build micro-judge
/usr/bin/docker-compose build micro-changelog
/usr/bin/docker-compose build micro-game-lobby
/usr/bin/docker-compose build micro-color-chooser
/usr/bin/docker-compose build micro-sync
/usr/bin/docker-compose build botlink
