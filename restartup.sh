#!/bin/bash

docker-compose down && \
    docker-compose up -d && \
    sleep 30 && \
    docker-compose restart game-lobby && \
    docker-compose restart color-chooser && \
    docker-compose restart history-provider && \
    sleep 15 && \
    docker-compose restart judge && \
    docker-compose restart changelog
