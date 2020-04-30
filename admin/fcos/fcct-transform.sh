#!/bin/sh

# Usage:
# 
# fcct-transform.sh < some.yaml > some.ign

docker run  -i --rm quay.io/coreos/fcct:release --pretty --strict
