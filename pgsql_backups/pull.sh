#!/bin/bash

# Check if .pgpass exists in current directory
if [ ! -f .pgpass ]; then
    echo "Error: .pgpass file not found in current directory"
    exit 1
fi

# Read database connection details from .pgpass
# Format: hostname:port:database:username:password
read -r DB_HOST DB_PORT DB_NAME DB_USER DB_PASS <<< $(head -1 .pgpass | tr ':' ' ')

# Set default PGPASSFILE to current directory's .pgpass
export PGPASSFILE="$(pwd)/.pgpass"

# Create backups directory if it doesn't exist
BACKUP_DIR="./backups"
mkdir -p "$BACKUP_DIR"

# Generate timestamp
TIMESTAMP=$(date +%s)

# Function to display help
show_help() {
    echo "Usage: $0 [OPTION]"
    echo "Options:"
    echo "  --schema    Pull only public schema (structure only)"
    echo "  --full      Pull full database (structure and data)"
    echo "  --help      Display this help message"
}

# Check arguments
if [ $# -eq 0 ]; then
    echo "Error: No option specified"
    show_help
    exit 1
fi

case $1 in
    --schema)
        BACKUP_TYPE="schema"
        OUTPUT_FILE="${BACKUP_DIR}/public_schema_${TIMESTAMP}.sql"
        # Pull schema only (no data)
        echo "Pulling public schema from database ${DB_NAME}..."
        pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
                --schema=public --schema-only --no-owner --no-privileges \
                -f "$OUTPUT_FILE"
        
        if [ $? -eq 0 ]; then
            echo "Schema saved to: $OUTPUT_FILE"
        else
            echo "Error: Failed to pull schema"
            exit 1
        fi
        ;;
        
    --full)
        BACKUP_TYPE="full"
        OUTPUT_FILE="${BACKUP_DIR}/public_full_${TIMESTAMP}.sql"
        # Pull full database
        echo "Pulling full database from ${DB_NAME}..."
        pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
                --schema=public --no-owner --no-privileges \
                -f "$OUTPUT_FILE"
        
        if [ $? -eq 0 ]; then
            echo "Full database saved to: $OUTPUT_FILE"
        else
            echo "Error: Failed to pull full database"
            exit 1
        fi
        ;;
        
    --help)
        show_help
        ;;
        
    *)
        echo "Error: Invalid option: $1"
        show_help
        exit 1
        ;;
esac