#!/bin/bash

./wait-for-it.sh kafka:9092 -s -- sleep 0 # TODO 10
gateway
