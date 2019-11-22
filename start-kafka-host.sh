#!/bin/bash

export HOSTNAME=$HOSTNAME
echo "Host $HOSTNAME"
docker-compose down && docker-compose -f dc-giant.yml up
