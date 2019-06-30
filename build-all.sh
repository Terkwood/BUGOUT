#!/bin/bash

sh build-judge.sh
sh build-gamestates-aggregator.sh
sh build-gateway.sh
docker-compose build kafkacat
