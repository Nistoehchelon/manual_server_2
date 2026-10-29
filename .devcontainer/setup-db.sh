#!/usr/bin/env bash
set -e

# Wait until PostgreSQL is ready (the feature starts it automatically)
until pg_isready -h localhost -p 5432 -U postgres; do
  sleep 1
done

# Create the user and database exactly as before
sudo -u postgres psql -c "CREATE USER user123 WITH PASSWORD 'password123';"
sudo -u postgres psql -c "CREATE DATABASE database123 OWNER user123;"

# Run your schema‑push script if it exists in the repo
if [ -f "pgsql_backups/push.sh" ]; then
  cd pgsql_backups
  chmod +x push.sh
  # The script likely expects to connect as user123; adjust as needed
  echo "yes" | bash push.sh --schema
  cd ..
fi

# Copy the example env file if you use one
[ -f .env.example ] && cp .env.example .env

echo "Database initialisation complete."