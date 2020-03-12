#!/bin/bash

cargo clean
docker build . -t tinybrain
