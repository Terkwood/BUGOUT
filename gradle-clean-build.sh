#!/bin/bash

cd $1 && gradle clean && cd - && docker-compose build $1

