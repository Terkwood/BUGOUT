#!/bin/bash


echo "This script will use sudo to perform two operations:"
echo "    sudo mkdir /mnt/stateful_partition/BUGOUT"
echo "    sudo chown $USER:$USER /mnt/stateful_partition/BUGOUT"
sudo mkdir /mnt/stateful_partition/BUGOUT
sudo chown $USER:$USER /mnt/stateful_partition/BUGOUT

mkdir /mnt/stateful_partition/BUGOUT/reverse-proxy

if [ `basename "$PWD"` == "BUGOUT" ]; then
   CADDYFILE_PATH="./reverse-proxy/Caddyfile.example.dev"
elif [ `basename "$PWD"` == "admin" ]; then
   CADDYFILE_PATH="../reverse-proxy/Caddyfile.example.dev"
else
    echo "Not sure where to find the example Caddyfile, exiting"
    exit 1
fi

cp $CADDYFILE_PATH /mnt/stateful_partition/BUGOUT/reverse-proxy/Caddyfile
