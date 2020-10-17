#!/bin/bash

sh cargo-clean-build.sh gateway
sh cargo-clean-build.sh micro-judge
sh cargo-clean-build.sh micro-changelog
sh compose.sh build micro-game-lobby
sh cargo-clean-build.sh micro-color-chooser
sh cargo-clean-build.sh micro-sync
