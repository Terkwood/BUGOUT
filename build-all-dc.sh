#!/bin/bash

alias docker-compose='docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v "$PWD:/rootfs/$PWD" \
  -w="/rootfs/$PWD" \
docker/compose:1.13.0'

# This script is useful for environments where you
# do not have access to cargo, gradle etc
docker-compose -f dc-giant.yml build judge
docker-compose -f dc-giant.yml  build changelog
docker-compose -f dc-tiny.yml  build gateway
docker-compose -f dc-giant.yml  build kafkacat
