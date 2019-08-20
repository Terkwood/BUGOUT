#!/bin/bash

cargo clean && docker cp . bugout_gateway_1:/var/BUGOUT/gateway/.
