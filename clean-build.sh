#!/bin/bash

cd $1 && cargo clean && cd - && docker-compose build $1
