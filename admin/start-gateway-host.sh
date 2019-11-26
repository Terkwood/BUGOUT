#!/bin/bash

cd /home/core/BUGOUT
/opt/bin/docker-compose down && /opt/bin/docker-compose -f dc-tiny.yml up -d
