#!/bin/bash

docker-compose -f dc-tiny.yml build gateway
docker-compose -f dc-tiny.yml build micro-judge
docker-compose -f dc-tiny.yml build micro-changelog
docker-compose -f dc-tiny.yml build bugle
docker-compose -f dc-tiny.yml build botlink
