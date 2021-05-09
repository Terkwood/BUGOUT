#!/bin/bash

export COMPOSE_DOCKER_CLI_BUILD=1 
export DOCKER_BUILDKIT=1
/usr/local/bin/docker-compose build gateway
/usr/local/bin/docker-compose build micro-judge
/usr/local/bin/docker-compose build micro-changelog
/usr/local/bin/docker-compose build micro-game-lobby
/usr/local/bin/docker-compose build micro-color-chooser
/usr/local/bin/docker-compose build micro-sync
/usr/local/bin/docker-compose build botlink
/usr/local/bin/docker-compose build undo
