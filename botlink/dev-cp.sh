#!/bin/bash

CT_ROOT=bugout_botlink_1:/var/BUGOUT/botlink

docker cp src/. $CT_ROOT/src/. && docker cp Cargo.toml $CT_ROOT/Cargo.toml
