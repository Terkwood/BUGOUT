#!/bin/bash

# Cheater script.  Use from inside the docker
# container, to reduce incremental build times

git fetch && git pull && cargo install --path . --force && RUST_BACKTRACE=1 gateway