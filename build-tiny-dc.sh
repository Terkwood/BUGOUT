#!/bin/bash

alias docker-compose='docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v "$PWD:/rootfs/$PWD" \
  -w="/rootfs/$PWD" \
docker/compose:1.13.0'


docker-compose -f dc-tiny.yml build gateway
docker-compose -f dc-tiny.yml build micro-judge
docker-compose -f dc-tiny.yml build micro-changelog
docker-compose -f dc-tiny.yml build bugle
docker-compose -f dc-tiny.yml build botlink
