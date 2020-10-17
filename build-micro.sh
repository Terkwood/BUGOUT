#!/bin/bash

sh cargo-clean-build.sh gateway
sh compose.sh build micro-judge
sh compose.sh build micro-changelog
sh compose.sh build micro-game-lobby
sh compose.sh build micro-color-chooser
sh compose.sh build micro-sync
