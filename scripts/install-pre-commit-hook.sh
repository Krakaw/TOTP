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

# Create .git/hooks directory if it doesn't exist
mkdir -p "$GIT_HOOKS_DIR"

# Check if there's an existing pre-commit hook
if [ -f "$PRE_COMMIT_HOOK" ] && [ ! -L "$PRE_COMMIT_HOOK" ]; then
    # It's a regular file (not a symlink), backup existing hook
    BACKUP="$PRE_COMMIT_HOOK.backup.$(date +%Y%m%d_%H%M%S)"
    echo "Backing up existing pre-commit hook to: $BACKUP"
    cp "$PRE_COMMIT_HOOK" "$BACKUP"
    
    # Create a wrapper that runs our hook, then the original
    cat > "$PRE_COMMIT_HOOK" << 'WRAPPER_EOF'
#!/bin/bash
# Auto-generated wrapper hook that chains to existing hooks

# Run the project's pre-commit hook
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
HOOK_SCRIPT="$PROJECT_ROOT/scripts/pre-commit"

if [ -f "$HOOK_SCRIPT" ]; then
    "$HOOK_SCRIPT" || exit 1
fi

# Run the original hook (if it exists and is different)
# Find the most recent backup
LATEST_BACKUP=$(ls -t "$SCRIPT_DIR/pre-commit.backup."* 2>/dev/null | head -n1)
if [ -n "$LATEST_BACKUP" ] && [ -f "$LATEST_BACKUP" ] && [ -x "$LATEST_BACKUP" ]; then
    "$LATEST_BACKUP" || exit 1
fi

exit 0
WRAPPER_EOF
    chmod +x "$PRE_COMMIT_HOOK"
    echo "Installed pre-commit hook (preserving existing hook as backup)."
    echo "The hook will run cargo fmt/clippy first, then your original hook."
elif [ -L "$PRE_COMMIT_HOOK" ]; then
    # It's already a symlink, check if it points to our script
    CURRENT_TARGET=$(readlink "$PRE_COMMIT_HOOK")
    
    # Resolve both to absolute paths for comparison
    ABSOLUTE_TARGET=$(cd "$GIT_HOOKS_DIR" && readlink -f "$PRE_COMMIT_HOOK" 2>/dev/null || echo "")
    ABSOLUTE_SCRIPT=$(readlink -f "$HOOK_SCRIPT" 2>/dev/null || echo "")
    
    if [ -n "$ABSOLUTE_TARGET" ] && [ -n "$ABSOLUTE_SCRIPT" ] && [ "$ABSOLUTE_TARGET" = "$ABSOLUTE_SCRIPT" ]; then
        echo "Pre-commit hook is already installed (symlink)."
        exit 0
    else
        echo "Warning: Pre-commit hook is a symlink pointing elsewhere."
        echo "Removing old symlink and creating new one."
        rm "$PRE_COMMIT_HOOK"
        # Use absolute path for reliability
        ln -s "$HOOK_SCRIPT" "$PRE_COMMIT_HOOK"
        echo "Pre-commit hook installed successfully (symlink)."
    fi
else
    # No existing hook, create a symlink to our script
    ln -s "$HOOK_SCRIPT" "$PRE_COMMIT_HOOK"
    echo "Pre-commit hook installed successfully (symlink)."
fi

