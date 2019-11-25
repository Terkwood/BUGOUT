#!/bin/bash

export HOSTNAME=$HOSTNAME
echo "Determine KAFKA_ADVERTISED_LISTENERS using env HOSTNAME: $HOSTNAME"
docker-compose down && docker-compose -f dc-giant.yml up
