#!/bin/bash

NAME=color-chooser

sh build.sh && docker cp bugout.$NAME.jar bugout_${NAME}_1:/home/gradle/.
