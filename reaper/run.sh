#!/bin/bash

./wait-for-it.sh kafka:9092 -s
reaper
