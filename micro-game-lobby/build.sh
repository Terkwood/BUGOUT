#!/bin/bash
DOCKER_BUILDKIT=1 docker build --build-arg BUILDKIT_INLINE_CACHE=1 -t bugout_micro-game-lobby:latest .
