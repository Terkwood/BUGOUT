#!/bin/bash


# Hack to run rtcwake from laptop via crontab

# needs to be run as root
# and YES you need the sudo, when it's in a crontab
# don't ask why
sudo rtcwake -m no -l -t $(date +%s -d 'today 22:57')
