#!/bin/bash
docker exec -it `docker ps|grep gateway|awk '{print $1;}'` bash
