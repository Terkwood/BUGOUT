#!/bin/bash

sh build-judge.sh
sh build-changelog.sh
sh build-history-provider.sh
sh build-gateway.sh
docker-compose build kafkacat
