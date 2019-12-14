#!/bin/sh

# Disable THP
echo never > /sys/kernel/mm/transparent_hugepage/enabled

# Enable kernel mod overcommit_memory
sysctl -w vm.overcommit_memory=1

# Raise TCP backlog setting
sysctl -w net.core.somaxconn=65535
