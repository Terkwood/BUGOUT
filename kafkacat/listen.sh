#!/bin/bash

kafkacat -b kafka:9092 -t $1 -C -K: 
