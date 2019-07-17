#!/bin/bash

PROJECT="gamestates-aggregator"
START_DIR=$(pwd)

cd $PROJECT && sh build.sh && cd .. && docker-compose build $PROJECT
