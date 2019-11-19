#!/bin/bash

docker-compose down && \
    docker-compose up -d && \
    echo "Sleep 60s" && \
    sleep 60 && \
    docker-compose restart game-lobby && \
    docker-compose restart color-chooser && \
    docker-compose restart history-provider && \
    docker-compose restart judge && \
    docker-compose restart changelog
