#!/bin/bash

alias docker-compose='docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v "$PWD:/rootfs/$PWD" \
  -w="/rootfs/$PWD" \
docker/compose:1.13.0'


docker-compose -f dc-giant.yml build judge
docker-compose -f dc-giant.yml build changelog
docker-compose -f dc-giant.yml build game-lobby
docker-compose -f dc-giant.yml build color-chooser
docker-compose -f dc-giant.yml build history-provider
docker-compose -f dc-giant.yml build participation
docker-compose -f dc-giant.yml build reaper
docker-compose -f dc-giant.yml build kafkacat
docker-compose -f dc-giant.yml build startup
