#!/bin/bash
docker stats --format "table {{.Name}}\t{{.MemUsage}}"
