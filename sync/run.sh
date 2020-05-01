#!/bin/bash

./wait-for-it.sh kafka:9092 -s
java -jar bugout.sync.jar
