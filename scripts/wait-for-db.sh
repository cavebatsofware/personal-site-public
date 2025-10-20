#!/bin/bash
# wait-for-db.sh - Wait for PostgreSQL to be ready before starting the application

set -e

host="${DATABASE_HOST:-localhost}"
port="${DATABASE_PORT:-5432}"
user="${POSTGRES_USER:-personal_site_user}"
db="${POSTGRES_DB:-personal_site}"
max_attempts="${DB_WAIT_MAX_ATTEMPTS:-30}"
attempt=1

echo "⏳ Waiting for PostgreSQL at $host:$port..."

until PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$host" -p "$port" -U "$user" -d "$db" -c '\q' 2>/dev/null; do
    if [ $attempt -eq $max_attempts ]; then
        echo "❌ PostgreSQL did not become ready in time"
        exit 1
    fi

    echo "⏳ Attempt $attempt/$max_attempts: PostgreSQL is unavailable - sleeping"
    attempt=$((attempt + 1))
    sleep 2
done

echo "✅ PostgreSQL is ready at $host:$port!"
