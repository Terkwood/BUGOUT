#!/bin/bash

sh build.sh && docker cp bugout.sync.jar bugout_sync_1:/home/gradle/.
