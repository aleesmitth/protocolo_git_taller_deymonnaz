#!/bin/bash
# IMPORTANT: remember to change the /etc/postgresql/<VERSION NUMBER>/main/pg_hba.conf file to use md5 instead of peer
# IMPORTANT: remember to have .env file with the variables defined in cargo root directory
# IMPORTANT: run chmod +x create_database.sh to allow execution of this file
# IMPORTANT: to execute this run ./create_database.sh

# Load environment variables from .env file
source .env
# Create the database if it doesn't exist
PGPASSWORD="$DB_PASSWORD" psql -U "$DB_USER" -h "$DB_HOST" -p "$DB_PORT" -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"

