#!/usr/bin/env bash

cockroach start-single-node \
         --insecure \
         --listen-addr=localhost:26257 \
         --http-addr=localhost:7999 \
         --store="etc/cockroach-data"