#!/bin/bash

./wait-for-it.sh kafka:9092 -s -- sleep 17
gateway
