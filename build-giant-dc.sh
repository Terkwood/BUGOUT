#!/bin/bash

docker-compose -f dc-giant.yml build judge
docker-compose -f dc-giant.yml build changelog
docker-compose -f dc-giant.yml build game-lobby
docker-compose -f dc-giant.yml build history-provider
docker-compose -f dc-giant.yml build kafkacat

