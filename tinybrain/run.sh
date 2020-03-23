#!/bin/bash

docker run -a stdin -a stdout -a stderr \
    -i --device=/dev/nvhost-ctrl \
    --device=/dev/nvhost-ctrl-gpu \
    --device=/dev/nvhost-prof-gpu \
    --device=/dev/nvmap --device=/dev/nvhost-gpu \
    --device=/dev/nvhost-as-gpu \
    -v /usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu -v /usr/local/cuda-10.0/targets/aarch64-linux/lib:/usr/local/cuda-10.0/targets/aarch64-linux/lib \
    -v ${PWD}/.env:/var/BUGOUT/.env \
    tinybrain
