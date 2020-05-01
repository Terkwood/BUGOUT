#!/bin/bash

export SUBNET_ID=subnet-deadbeef
export VPC_ID=vpc-foobar
packer validate some-packer.json
ENV_SRC=dev-env USER_DATA_FILE=some.ign \
	packer build some-packer.json
