#!/bin/bash

sh build-judge.sh
sh build-gamestates-aggregator.sh
docker-compose build kafkacat
