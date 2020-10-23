#!/bin/bash

sh build.sh && docker cp bugout.changelog.jar bugout_changelog_1:/home/gradle/.
