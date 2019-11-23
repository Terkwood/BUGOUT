#!/bin/bash

docker-compose restart game-lobby && \
docker-compose restart color-chooser && \
docker-compose restart history-provider && \
docker-compose restart judge && \
docker-compose restart changelog
