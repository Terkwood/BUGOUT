#!/bin/bash

sh compose.sh build gateway
sh compose.sh build micro-judge
sh compose.sh build micro-changelog
sh compose.sh build micro-game-lobby
sh compose.sh build micro-color-chooser
sh compose.sh build micro-sync
sh compose.sh build botlink

