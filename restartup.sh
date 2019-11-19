#!/bin/bash

docker-compose down && \
    docker-compose up -d && \
    echo "Sleep 30s" && \
    sleep 30 && \
    docker-compose restart game-lobby && \
    docker-compose restart color-chooser && \
    docker-compose restart history-provider && \
    echo "Sleep 15s" && \
    sleep 15 && \
    docker-compose restart judge && \
    docker-compose restart changelog
