#!/bin/bash

export COMPOSE_DOCKER_CLI_BUILD=1 
export DOCKER_BUILDKIT=1
docker-compose build gateway
docker-compose build micro-judge
docker-compose build micro-changelog
docker-compose build micro-game-lobby
docker-compose build micro-color-chooser
docker-compose build micro-sync
docker-compose build botlink
