#!/bin/bash

PROJECT="gateway"
START_DIR=$(pwd)

cd $PROJECT && cargo clean && cd .. && docker-compose build $PROJECT
