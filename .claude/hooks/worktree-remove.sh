#!/usr/bin/env bash
set -euo pipefail

INPUT=$(cat)
WORKTREE_PATH=$(echo "$INPUT" | jq -r '.worktree_path')

MAIN_REPO=$(git -C "$WORKTREE_PATH" rev-parse --path-format=absolute --git-common-dir)
MAIN_REPO="${MAIN_REPO%/.git}"

git -C "$MAIN_REPO" worktree remove --force "$WORKTREE_PATH" >&2
