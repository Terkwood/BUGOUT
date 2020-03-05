#!/bin/bash

sh gradle-clean-build.sh judge
sh gradle-clean-build.sh changelog
sh gradle-clean-build.sh history-provider
sh gradle-clean-build.sh game-lobby
sh gradle-clean-build.sh color-chooser
sh gradle-clean-build.sh participation
docker-compose build kafkacat
sh cargo-clean-build.sh gateway
sh cargo-clean-build.sh micro-judge
sh cargo-clean-build.sh micro-changelog
sh cargo-clean-build.sh bugle
docker-compose build startup
