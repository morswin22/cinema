#!/bin/bash
set -e

# Wait for Mysql to be ready
until mysqladmin ping -h db -u user -ppassword --silent; do
  echo "Waiting for Mysql to be ready..."
  sleep 2
done

# Set up the DB and run migrations
#diesel setup --database-url="$DATABASE_URL"

# Optional: generate a migration based on schema diff
#diesel migration generate --diff-schema create_tables

diesel migration run

diesel print-schema

# Keep container alive or run your app here
cargo run
