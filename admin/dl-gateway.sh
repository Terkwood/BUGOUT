#!/bin/bash

# Show logs for gateway container

docker ps|grep gateway|awk '{print $1;}'|xargs docker logs -f
