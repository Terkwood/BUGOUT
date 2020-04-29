#!/bin/bash

export HOSTNAME=$HOSTNAME.ec2.internal
echo "Determine KAFKA_ADVERTISED_LISTENERS using env HOSTNAME: $HOSTNAME"
cd /home/core/BUGOUT
/opt/bin/docker-compose down && /opt/bin/docker-compose -f dc-giant.yml up -d
