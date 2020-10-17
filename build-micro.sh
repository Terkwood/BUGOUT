#!/bin/bash

sh cargo-clean-build.sh gateway
sh cargo-clean-build.sh micro-judge
sh cargo-clean-build.sh micro-changelog
sh compose.sh build micro-game-lobby
sh compose.sh micro-color-chooser
sh compose.sh micro-sync
