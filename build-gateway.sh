#!/bin/bash

PROJECT="gateway"
START_DIR=$(pwd)

cd $PROJECT && cargo clean && cd .. && docker-compose -f dc-tiny.yml build $PROJECT
