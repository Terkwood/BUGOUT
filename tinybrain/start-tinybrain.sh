#!/bin/bash

# systemd can use this script to start tinybrain

if [ -z "$TINYBRAIN_HOME" ]
then
	echo "Please specify TINYBRAIN_HOME in env"
	exit 1
fi

if [ -z "$TINYBRAIN_BIN" ]
then
	echo "Please specify TINYBRAIN_BIN (parent folder of executable) in env"
	exit 1
fi

export PATH="$PATH:$TINYBRAIN_BIN"

# sorry, but we want to give the network additional
# time to sort itself out.
sleep 20 

# move to a location where we expect a .env
# file and a compiled katago executable
cd $TINYBRAIN_HOME
tinybrain
