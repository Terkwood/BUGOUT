#!/bin/bash

cd gateway && cargo clean && cd - && docker-compose build gateway
