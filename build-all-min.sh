#!/bin/bash

# This script is useful for environments where you
# do not have access to cargo, gradle etc
docker-compose build judge
docker-compose build gamestates-aggregator
docker-compose build gateway
docker-compose build kafkacat
