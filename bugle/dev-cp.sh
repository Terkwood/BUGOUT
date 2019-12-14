#!/bin/bash


docker cp Cargo.toml bugout_bugle_1:/var/BUGOUT/bugle/.
docker cp src bugout_bugle_1:/var/BUGOUT/bugle/.
docker cp .env bugout_bugle_1:/var/BUGOUT/bugle/.env

