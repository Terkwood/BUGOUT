#!/bin/bash

export HOSTNAME=$HOSTNAME
echo "Determine KAFKA_ADVERTISED_LISTENERS using env HOSTNAME: $HOSTNAME"
/opt/bin/docker-compose down && /opt/bin/docker-compose -f /home/core/BUGOUT/dc-giant.yml up
