#!/bin/bash

docker-compose -f dc-tiny.yml --env-file gateway/.env build gateway
