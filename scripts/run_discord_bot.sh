#!/usr/bin/env bash

source .env
if [ -f etc/.env ]; then
  source etc/.env
fi

export DATABASE_URL=$DISCORD_DATABASE_URL && export DISCORD_TOKEN=$DISCORD_TOKEN && cargo run -p discord-tickets