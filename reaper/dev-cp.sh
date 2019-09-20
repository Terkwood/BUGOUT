#!/bin/bash

docker cp Cargo.toml bugout_reaper_1:/var/BUGOUT/reaper/.
docker cp src bugout_reaper_1:/var/BUGOUT/reaper/src
docker cp examples bugout_reaper_1:/var/BUGOUT/reaper/examples

