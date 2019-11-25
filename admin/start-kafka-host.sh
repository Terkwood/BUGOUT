#!/bin/bash

export HOSTNAME=$HOSTNAME
echo "Determine KAFKA_ADVERTISED_LISTENERS using env HOSTNAME: $HOSTNAME"
cd /home/core/BUGOUT
echo $PWD
/opt/bin/docker-compose down && /opt/bin/docker-compose -f dc-giant.yml up
