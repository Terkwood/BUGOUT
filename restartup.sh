#!/bin/bash

docker-compose down && \
    docker-compose up && \
    docker-compose restart judge && \
    docker-compose restart changelog && \
    docker-compose restart game-lobby && \
    docker-compose restart color-chooser && \
    docker-compose restart history-provider
