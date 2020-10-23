#!/bin/bash

./wait-for-it.sh kafka:9092 -s
java -jar bugout.color-chooser.jar
