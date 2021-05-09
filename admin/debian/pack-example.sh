#!/bin/bash

set -e

export SSH_PUBKEY=`echo ~/.ssh/id_rsa.pub`
export VPC_ID="vpc-bbbbbbbb"
export SUBNET_ID="subnet-aaaaaaaa"
export DOTENV_SRC="./some-bugout-folder"
packer validate packer.json && \
  packer build packer.json

