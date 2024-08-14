#!/bin/bash

entities_dir="./database/entities/src/"

source .env
sea-orm-cli generate entity \
    --database-url postgresql://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME \
    --output-dir $entities_dir \
#    --model-extra-derives 'schemars::JsonSchema' \
    --with-serde both \
    --lib