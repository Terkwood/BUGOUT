#!/bin/bash

./wait-for-it.sh kafka:9092 -s -- sleep 10 
java -jar bugout.changelog.jar
