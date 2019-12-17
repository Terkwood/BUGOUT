#!/bin/bash

mkdir -p /opt/bin
LATEST_URL=`curl -Ls -o /dev/null -w %{url_effective} https://github.com/docker/compose/releases/latest`
COMPOSE_VERSION=${LATEST_URL##*/}
DOWNLOAD_URL=https://github.com/docker/compose/releases/download/${COMPOSE_VERSION}/docker-compose-`uname -s`-`uname -m`

curl -L ${DOWNLOAD_URL} -o /opt/bin/docker-compose
chmod +x /opt/bin/docker-compose
