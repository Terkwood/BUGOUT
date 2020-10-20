#!/bin/bash

export COMPOSE_DOCKER_CLI_BUILD=1 
export DOCKER_BUILDKIT=1
/opt/bin/docker-compose build gateway
/opt/bin/docker-compose build micro-judge
/opt/bin/docker-compose build micro-changelog
/opt/bin/docker-compose build micro-game-lobby
/opt/bin/docker-compose build micro-color-chooser
/opt/bin/docker-compose build micro-sync
/opt/bin/docker-compose build botlink
