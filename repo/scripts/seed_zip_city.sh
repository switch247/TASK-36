#!/usr/bin/env sh
set -e

MYSQL_PWD="$MYSQL_PASSWORD" mysql -h "$MYSQL_HOST" -P "$MYSQL_PORT" -u "$MYSQL_USER" "$MYSQL_DATABASE" < app/models/migrations/002_seed_zip_city.sql
