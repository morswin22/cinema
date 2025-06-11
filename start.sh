#!/bin/bash
set -e

until mysqladmin ping -h mysqld -u user -ppassword --silent; do
  echo "Waiting for Mysql to be ready..."
  sleep 2
done

if [ -n "${MIGRATION}" ]; then
  diesel migration run
  diesel print-schema
else
  cargo run --bin Cinema
fi
