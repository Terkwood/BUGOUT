#!/bin/bash
docker run  -i --rm quay.io/coreos/fcct:release --pretty --strict <  $1.yaml > $1.ign

