#!/bin/bash

if [ -z "$TINYBRAIN_USER" ]
then
	echo "Please specify TINYBRAIN_USER in env"
	exit 1
fi

# move to a location where we expect a .env
# file and a compiled katago executable
cd /home/$TINYBRAIN_USER/BUGOUT/tinybrain
tinybrain
