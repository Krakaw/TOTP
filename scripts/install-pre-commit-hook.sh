#!/bin/bash

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"
PRE_COMMIT_HOOK="$GIT_HOOKS_DIR/pre-commit"
HOOK_SCRIPT="$SCRIPT_DIR/pre-commit"

# Check if we're in a git repository
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "Error: Not a git repository" >&2
    exit 1
fi

# Check if a global or local hooksPath is set
CURRENT_HOOKS_PATH=$(git config --get core.hooksPath 2>/dev/null || echo "")
LOCAL_HOOKS_PATH=$(git config --local --get core.hooksPath 2>/dev/null || echo "")

# Set local hooksPath to point to scripts directory to override global setting
# This ensures our hooks are found even if a global hooksPath is configured
if [ "$LOCAL_HOOKS_PATH" != "$SCRIPT_DIR" ]; then
    git config --local core.hooksPath "$SCRIPT_DIR"
    if [ -n "$CURRENT_HOOKS_PATH" ] && [ "$CURRENT_HOOKS_PATH" != "$SCRIPT_DIR" ]; then
        echo "Note: Global hooksPath ($CURRENT_HOOKS_PATH) is overridden by local setting ($SCRIPT_DIR)"
    fi
    echo "Set local hooksPath to: $SCRIPT_DIR"
fi

# Create .git/hooks directory if it doesn't exist (for backward compatibility)
mkdir -p "$GIT_HOOKS_DIR"

# Check if pre-commit hook already exists in scripts directory
if [ -f "$HOOK_SCRIPT" ] && [ -x "$HOOK_SCRIPT" ]; then
    echo "Pre-commit hook is ready in: $HOOK_SCRIPT"
    echo "Git will use this hook via hooksPath configuration."
else
    echo "Error: Pre-commit hook script not found at: $HOOK_SCRIPT" >&2
    exit 1
fi

# Check if there's an existing pre-commit hook in the old .git/hooks location
# (for backward compatibility, but it won't be used if hooksPath is set)
if [ -f "$PRE_COMMIT_HOOK" ] && [ ! -L "$PRE_COMMIT_HOOK" ]; then
    echo "Warning: Found pre-commit hook in .git/hooks/ (old location)"
    echo "This won't be used since hooksPath is set. Consider removing it."
elif [ -L "$PRE_COMMIT_HOOK" ]; then
    echo "Note: Found symlink in .git/hooks/ (old location)"
    echo "This won't be used since hooksPath is set. You can remove it if desired."
fi

echo "Pre-commit hook installation complete!"
echo "The hook will run cargo fmt/clippy, then any global hooks."

