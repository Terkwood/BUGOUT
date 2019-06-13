#!/bin/bash

rm -f ./build/libs/bugout.gamestates.aggregator*.jar
gradle build
cp ./build/libs/bugout.gamestates.aggregator-*.jar bugout.gamestates.aggregator.jar
