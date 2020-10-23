#!/bin/bash

sh build.sh && docker cp bugout.judge.jar bugout_judge_1:/home/gradle/.
