#!/bin/bash

set -e

export VPC_ID="vpc-bbbbbbbb"
export SUBNET_ID="subnet-aaaaaaaa"
export DOTENV_SRC="./some-bugout-folder"
packer validate packer.json && \
  packer build packer.json

