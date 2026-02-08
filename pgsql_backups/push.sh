#!/bin/bash

# Check if .pgpass exists in current directory
if [ ! -f .pgpass ]; then
    echo "Error: .pgpass file not found in current directory"
    exit 1
fi

# Read database connection details from .pgpass
read -r DB_HOST DB_PORT DB_NAME DB_USER DB_PASS <<< $(head -1 .pgpass | tr ':' ' ')

# Set default PGPASSFILE to current directory's .pgpass
export PGPASSFILE="$(pwd)/.pgpass"

BACKUP_DIR="./backups"

# Function to display help
show_help() {
    echo "Usage: $0 [OPTION] [FILENAME]"
    echo "Options:"
    echo "  --schema    Push schema backup (wipe and import)"
    echo "  --full      Push full backup (wipe and import)"
    echo "  --help      Display this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --schema                      # Uses latest schema backup"
    echo "  $0 --full public_full_12345.sql  # Uses specified file"
}

# Function to check if filename matches the expected pattern
check_filename_pattern() {
    local filename=$1
    local expected_type=$2
    
    case $expected_type in
        schema)
            if [[ "$filename" != public_schema_* ]]; then
                echo "Error: File '$filename' does not match schema pattern (public_schema_*)"
                exit 1
            fi
            ;;
        full)
            if [[ "$filename" != public_full_* ]]; then
                echo "Error: File '$filename' does not match full pattern (public_full_*)"
                exit 1
            fi
            ;;
    esac
}

# Function to extract timestamp from filename
get_timestamp() {
    local filename=$1
    # Extract numbers after the last underscore
    echo "$filename" | grep -oE '[0-9]+' | tail -1
}

# Check arguments
if [ $# -eq 0 ]; then
    echo "Error: No option specified"
    show_help
    exit 1
fi

case $1 in
    --schema|--full)
        BACKUP_TYPE="${1#--}"  # Remove '--' prefix
        PREFIX="public_${BACKUP_TYPE}_"
        
        # Check if backup directory exists
        if [ ! -d "$BACKUP_DIR" ]; then
            echo "Error: Backup directory '$BACKUP_DIR' not found"
            exit 1
        fi
        
        # Determine which file to use
        if [ $# -eq 2 ]; then
            # Use specified file
            BACKUP_FILE="$2"
            # Check if file exists
            if [ ! -f "${BACKUP_DIR}/${BACKUP_FILE}" ]; then
                echo "Error: File '${BACKUP_DIR}/${BACKUP_FILE}' not found"
                exit 1
            fi
            # Check filename pattern
            check_filename_pattern "$BACKUP_FILE" "$BACKUP_TYPE"
        else
            # Find the latest file based on timestamp
            echo "Looking for latest ${BACKUP_TYPE} backup..."
            
            # List files with the pattern, sort by timestamp, and get the latest
            LATEST_FILE=$(ls "${BACKUP_DIR}/${PREFIX}"*.sql 2>/dev/null | \
                         sort -t '_' -k3 -nr | head -1)
            
            if [ -z "$LATEST_FILE" ]; then
                echo "Error: No ${BACKUP_TYPE} backup files found in ${BACKUP_DIR}"
                exit 1
            fi
            
            BACKUP_FILE=$(basename "$LATEST_FILE")
            echo "Selected: $BACKUP_FILE"
        fi
        
        FULL_PATH="${BACKUP_DIR}/${BACKUP_FILE}"
        
        # Wipe and import
        echo "Pushing ${BACKUP_TYPE} backup to database ${DB_NAME}..."
        echo "Warning: This will wipe the public schema!"
        read -p "Are you sure? (yes/no): " CONFIRM
        
        if [ "$CONFIRM" != "yes" ]; then
            echo "Operation cancelled"
            exit 0
        fi
        
        # Drop all tables in public schema
        echo "Dropping all tables in public schema..."
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
             -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
        
        # Import the backup
        echo "Importing ${BACKUP_FILE}..."
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
             -f "$FULL_PATH"
        
        if [ $? -eq 0 ]; then
            echo "Successfully imported ${BACKUP_FILE}"
        else
            echo "Error: Failed to import backup"
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