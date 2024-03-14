#!/usr/bin/env bash

source .env
if [ -f etc/.env ]; then
  source etc/.env
fi

echo $COLLECTOR_DATABASE_URL

export DATABASE_URL=$COLLECTOR_DATABASE_URL && cargo run -p collector