#!/bin/bash

/opt/bin/docker-compose -f dc-giant.yml build judge
/opt/bin/docker-compose -f dc-giant.yml build changelog
/opt/bin/docker-compose -f dc-giant.yml build game-lobby
/opt/bin/docker-compose -f dc-giant.yml build color-chooser
/opt/bin/docker-compose -f dc-giant.yml build history-provider
/opt/bin/docker-compose -f dc-giant.yml build participation
/opt/bin/docker-compose -f dc-giant.yml build reaper
/opt/bin/docker-compose -f dc-giant.yml build kafkacat
/opt/bin/docker-compose -f dc-giant.yml build startup
/opt/bin/docker-compose -f dc-giant.yml build sync
