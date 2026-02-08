#!/bin/bash

# Directory containing backups
BACKUP_DIR="./backups"

# Check if backups directory exists
if [ ! -d "$BACKUP_DIR" ]; then
    echo "Error: Backup directory '$BACKUP_DIR' does not exist."
    exit 1
fi

# Change to backup directory
cd "$BACKUP_DIR" || exit 1

# clean_old_backups.sh - Alternative version with proper timestamp extraction

# Function to process backup files of a specific type
process_backup_type() {
    local backup_type="$1"  # "full" or "schema"
    
    echo "Processing $backup_type backups..."
    
    # Create an array to store files with their timestamps
    declare -a file_array
    
    # Read all matching files
    for file in public_${backup_type}_*.sql; do
        # Skip if no files match the pattern
        [[ -f "$file" ]] || continue
        
        # Extract timestamp from filename
        timestamp=$(echo "$file" | grep -oE '[0-9]+' | head -1)
        
        if [[ -n "$timestamp" ]]; then
            # Store filename and timestamp
            file_array+=("$timestamp:$file")
        fi
    done
    
    local count=${#file_array[@]}
    echo "Found $count ${backup_type} backups"
    
    if [ $count -eq 0 ]; then
        echo "No ${backup_type} backups found"
        echo ""
        return
    fi
    
    # Sort by timestamp descending (newest first)
    IFS=$'\n' sorted_files=($(sort -t: -k1 -rn <<<"${file_array[*]}"))
    unset IFS
    
    if [ $count -gt 3 ]; then
        echo "Keeping the 3 newest ${backup_type} backups, removing older ones..."
        
        # Extract just filenames, skipping timestamps
        for ((i=0; i<count; i++)); do
            if [ $i -lt 3 ]; then
                # Keep first 3 files (newest)
                filename=$(echo "${sorted_files[i]}" | cut -d: -f2)
                echo "Keeping: $filename"
            else
                # Remove older files
                filename=$(echo "${sorted_files[i]}" | cut -d: -f2)
                echo "Removing: $filename"
                rm "$filename"
            fi
        done
        echo "Removed $((count - 3)) old ${backup_type} backups"
    else
        echo "No cleanup needed for ${backup_type} backups (3 or fewer found)"
        # List all files
        for ((i=0; i<count; i++)); do
            filename=$(echo "${sorted_files[i]}" | cut -d: -f2)
            echo "  $filename"
        done
    fi
    
    echo ""
}

# Main script execution
echo "Starting backup cleanup..."
echo "=========================="

# Process full backups
process_backup_type "full"

# Process schema backups  
process_backup_type "schema"

echo "Cleanup completed!"