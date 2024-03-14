#!/usr/bin/env bash

source .env
if [ -f etc/.env ]; then
  source etc/.env
fi

for db in \
  $COLLECTOR_DATABASE_URL,collector/migrations/ \
  $DISCORD_DATABASE_URL,gateways/discord/migrations/ \
; do
  IFS=","; set -- $db
  echo -e "Dropping database $1\n"
  cargo sqlx database drop --database-url $1 -y
  echo -e "Creating database $1\n with migrations from $2"
  cargo sqlx database setup --database-url $1 --source $2
  echo -e ""
done
echo -e "Migrations and database creation complete."
