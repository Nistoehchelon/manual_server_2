gitacp() {
    if [ $# -eq 0 ]; then
        echo "Usage: gitacp \"commit message\""
        return 1
    fi

    # Stage changes (tracked + untracked)
    git add -A

    # Commit with proper quoting
    git commit -m "$*"
    if [ $? -ne 0 ]; then
        echo "Commit failed. Aborting push."
        return 1
    fi

    # Push to current branch
    git push
}

