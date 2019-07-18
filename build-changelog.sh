#!/bin/bash

PROJECT="changelog"
START_DIR=$(pwd)

cd $PROJECT && sh build.sh && cd .. && docker-compose build $PROJECT
