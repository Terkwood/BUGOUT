#!/bin/bash

PROJECT="judge"
START_DIR=$(pwd)

cd $PROJECT && sh build.sh && cd .. && docker-compose build $PROJECT
