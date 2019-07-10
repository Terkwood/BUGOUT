#!/bin/bash

# Cheater script.  Use from inside the docker
# container, to reduce incremental build times

git fetch && git pull 
git rev-parse HEAD
cargo install --path . --force 
gateway
