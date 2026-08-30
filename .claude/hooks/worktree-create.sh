#!/usr/bin/env bash
set -euo pipefail

INPUT=$(cat)
NAME=$(echo "$INPUT" | jq -r '.name')
CWD=$(echo "$INPUT" | jq -r '.cwd')

REPO_ROOT=$(git -C "$CWD" rev-parse --show-toplevel)
WORKTREE_DIR="$REPO_ROOT/.worktrees/$NAME"

cleanup() {
    if [ -d "$WORKTREE_DIR" ]; then
        git -C "$REPO_ROOT" worktree remove --force "$WORKTREE_DIR" 2>/dev/null || true
    fi
}
trap cleanup ERR

mkdir -p "$(dirname "$WORKTREE_DIR")"
git -C "$REPO_ROOT" worktree add "$WORKTREE_DIR" >&2

cd "$WORKTREE_DIR"
pnpm install >&2

echo "$WORKTREE_DIR"
