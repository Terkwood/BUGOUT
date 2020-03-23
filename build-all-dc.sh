#!/bin/bash

alias docker-compose='docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v "$PWD:/rootfs/$PWD" \
  -w="/rootfs/$PWD" \
docker/compose:1.13.0'

# This script is useful for environments where you
# do not have access to cargo, gradle etc
docker-compose build judge
docker-compose build changelog
docker-compose build history-provider
docker-compose build game-lobby
docker-compose build color-chooser
docker-compose build gateway
docker-compose build startup
docker-compose build reaper
docker-compose build bugle
docker-compose build kafkacat
docker-compose build micro-judge
docker-compose build micro-changelog
docker-compose build botlink
