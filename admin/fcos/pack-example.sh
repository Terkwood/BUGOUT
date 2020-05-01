#!/bin/bash

export VPC_ID="vpc-bbbbbbbb"
export SUBNET_ID="subnet-aaaaaaaa"
export DOTENV_SRC="./some-bugout-folder"
export USER_DATA_FILE="some-gateway.ign"
packer validate gateway-packer.json && \
  packer build gateway-packer.json

