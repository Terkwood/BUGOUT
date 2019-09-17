#!/bin/bash

./wait-for-it.sh kafka:9092 -s -- sleep 16

#TODO below
#reaper

cargo run --example shutdown
