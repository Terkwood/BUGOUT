#!/bin/bash

CT_ROOT=bugout_gateway_1:/var/BUGOUT/gateway

docker cp src/. $CT_ROOT/src/. && docker cp Cargo.toml $CT_ROOT/Cargo.toml
